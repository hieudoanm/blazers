use super::*;

pub async fn list_buckets(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Bucket>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let buckets = db::list_buckets(&conn)?;
    Ok(Json(buckets))
}

pub async fn create_bucket(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateBucketRequest>,
) -> std::result::Result<Json<Bucket>, AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let bucket = db::insert_bucket(&conn, &req.name, req.is_public)?;
    let dir = state.storage_dir.join("storage").join(&req.name);
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Internal(format!("create storage dir: {e}")))?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_BUCKET_CREATE,
        webhook::webhook_bucket_data(&bucket),
    );
    Ok(Json(bucket))
}

pub async fn get_bucket(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Json<Bucket>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let bucket =
        db::get_bucket(&conn, &name)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(bucket))
}

pub async fn delete_bucket(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_bucket(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let b = existing.unwrap();
    webhook::dispatch_event(
        &state,
        webhook::EVENT_BUCKET_DELETE,
        webhook::webhook_bucket_data(&b),
    );
    let files = db::delete_bucket(&conn, &name)?;
    let dir = state.storage_dir.join("storage").join(&name);
    for f in &files {
        let _ = std::fs::remove_file(state.storage_dir.join("storage").join(&name).join(&f.id));
    }
    let _ = std::fs::remove_dir_all(&dir);
    Ok(StatusCode::NO_CONTENT)
}
