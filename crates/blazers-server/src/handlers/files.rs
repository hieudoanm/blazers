use super::*;

pub async fn upload_file(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(bucket): Path<String>,
    req: axum::extract::Request,
) -> std::result::Result<Json<FileRecord>, AppError> {
    extract_claims(&headers)?;

    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .ok_or_else(|| AppError::BadRequest("content-type header required".into()))?
        .to_str()
        .map_err(|_| AppError::BadRequest("invalid content-type".into()))?
        .to_string();

    let boundary = multer::parse_boundary(&content_type)
        .map_err(|_| AppError::BadRequest("could not parse multipart boundary".into()))?;

    {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let existing = db::get_bucket(&conn, &bucket)?;
        if existing.is_none() {
            return Err(AppError::NotFound("bucket not found".into()));
        }
    }

    let stream = http_body_util::BodyDataStream::new(req.into_body());
    let mut multipart = multer::Multipart::new(stream, boundary);

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
        .ok_or_else(|| AppError::BadRequest("no file field in multipart".into()))?;

    let file_name = field.file_name().unwrap_or("file").to_string();
    let mut mime_type = field
        .content_type()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("read file error: {e}")))?;

    if data.len() > 10 * 1024 * 1024 {
        return Err(AppError::BadRequest("file too large, max 10MB".into()));
    }

    if data.is_empty() {
        return Err(AppError::BadRequest("file is empty".into()));
    }

    if mime_type == "application/octet-stream"
        && let Some(t) = infer::get(&data)
    {
        mime_type = t.mime_type().to_string();
    }

    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let file_dir = state.storage_dir.join("storage").join(&bucket);
    std::fs::create_dir_all(&file_dir)
        .map_err(|e| AppError::Internal(format!("create storage dir: {e}")))?;
    let file_path = file_dir.join(&id);
    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|e| AppError::Internal(format!("write file: {e}")))?;

    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let record = db::insert_file(
        &conn,
        &bucket,
        &id,
        &file_name,
        &mime_type,
        data.len() as i64,
    )?;
    Ok(Json(record))
}

pub async fn list_files(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(bucket): Path<String>,
    Query(params): Query<PaginationParams>,
) -> std::result::Result<Json<FilesPage>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_bucket(&conn, &bucket)?;
    if existing.is_none() {
        return Err(AppError::NotFound("bucket not found".into()));
    }
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let files = db::list_files(&conn, &bucket, page, per_page)?;
    Ok(Json(files))
}

pub async fn download_file(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((bucket, file_id)): Path<(String, String)>,
) -> std::result::Result<axum::response::Response<Body>, AppError> {
    extract_claims(&headers)?;

    let f = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let f =
            db::get_file(&conn, &file_id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
        if f.bucket != bucket {
            return Err(AppError::NotFound("not found".into()));
        }
        f
    };

    let file_path = state
        .storage_dir
        .join("storage")
        .join(&bucket)
        .join(&file_id);
    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|_| AppError::NotFound("file not found on disk".into()))?;

    let response = axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, &f.mime_type)
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", f.filename),
        )
        .header(axum::http::header::CONTENT_LENGTH, f.size.to_string())
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(response)
}

pub async fn delete_file(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((bucket, file_id)): Path<(String, String)>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let f = db::get_file(&conn, &file_id)?;
    match f {
        Some(ref f) if f.bucket == bucket => {}
        _ => return Err(AppError::NotFound("not found".into())),
    }
    let deleted = db::delete_file(&conn, &file_id)?;
    if deleted.is_some() {
        let file_path = state
            .storage_dir
            .join("storage")
            .join(&bucket)
            .join(&file_id);
        let _ = std::fs::remove_file(&file_path);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_thumbnail(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((bucket, file_id)): Path<(String, String)>,
) -> std::result::Result<axum::response::Response<Body>, AppError> {
    extract_claims(&headers)?;
    let f = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let f =
            db::get_file(&conn, &file_id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
        if f.bucket != bucket {
            return Err(AppError::NotFound("not found".into()));
        }
        f
    };
    if !crate::image::is_supported_image(&f.mime_type) {
        return Err(AppError::BadRequest("unsupported image type".into()));
    }
    let file_path = state
        .storage_dir
        .join("storage")
        .join(&bucket)
        .join(&file_id);
    let thumb_data =
        crate::image::generate_thumbnail(&file_path, 256).map_err(AppError::Internal)?;
    let response = axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "image/jpeg")
        .body(Body::from(thumb_data))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(response)
}
