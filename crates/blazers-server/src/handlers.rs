use std::path::PathBuf;

use axum::{
    Json,
    body::Body,
    extract::ws::Message,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::auth;
use crate::notification;
use crate::openapi;
use crate::pubsub;
use crate::rbac;
use crate::secrets;
use crate::webhook;
use crate::websocket;
use blazers_core::db;
use blazers_core::models::*;

pub async fn get_openapi_json() -> impl axum::response::IntoResponse {
    let spec = openapi::build_openapi_json();
    let json_str = serde_json::to_string_pretty(&spec).unwrap_or_default();
    axum::response::Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json_str))
        .unwrap()
}

pub async fn get_swagger_ui() -> impl axum::response::IntoResponse {
    axum::response::Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Blazers Server API – Swagger UI</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({ url: '/api/openapi.json', dom_id: '#swagger-ui' });
  </script>
</body>
</html>"#,
        ))
        .unwrap()
}

pub struct AppState {
    pub db: deadpool::managed::Pool<db::ConnectionManager>,
    pub storage_dir: PathBuf,
    pub http_client: reqwest::Client,
    pub secrets_key: Vec<u8>,
    pub ws_hub: std::sync::Arc<tokio::sync::RwLock<websocket::WSHub>>,
    pub cache: std::sync::Arc<crate::cache::CacheStore>,
    pub sse_hub: std::sync::Arc<tokio::sync::RwLock<crate::notification::SSEHub>>,
    pub log_hub: std::sync::Arc<tokio::sync::RwLock<crate::log::SSEHub>>,
    pub pubsub_hub: std::sync::Arc<tokio::sync::RwLock<crate::pubsub::SSEHub>>,
    pub rate_limiter: std::sync::Arc<crate::rate_limit::RateLimiter>,
}

pub(crate) fn extract_token(headers: &HeaderMap) -> std::result::Result<String, AppError> {
    let auth = headers
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("authorization header required".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("invalid authorization header".into()))?;
    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("bearer token required".into()))?;
    Ok(token.to_string())
}

pub(crate) fn extract_claims(headers: &HeaderMap) -> std::result::Result<auth::Claims, AppError> {
    let token = extract_token(headers)?;
    auth::validate_token(&token)
}

pub async fn get_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn get_backup(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<(HeaderMap, Vec<u8>), AppError> {
    let claims = extract_claims(&headers)?;
    {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        rbac::require_role(&conn, &claims.user_id, "*", "admin")?;
    }
    let path = std::env::temp_dir().join(format!("blazers-backup-{}.db", uuid::Uuid::new_v4()));
    {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute("VACUUM INTO ?1", rusqlite::params![path.to_str().unwrap()])
            .map_err(|e| AppError::Internal(format!("backup: {e}")))?;
    }
    let data = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::Internal(format!("read backup: {e}")))?;
    tokio::fs::remove_file(&path).await.ok();
    let headers = axum::http::HeaderMap::from_iter([
        (
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/octet-stream"),
        ),
        (
            axum::http::header::CONTENT_DISPOSITION,
            axum::http::HeaderValue::from_str("attachment; filename=\"blazers-backup.db\"")
                .map_err(|e| AppError::Internal(e.to_string()))?,
        ),
    ]);
    Ok((headers, data))
}

pub async fn post_register(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> std::result::Result<Json<UserResponse>, AppError> {
    if req.email.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest(
            "email and password are required".into(),
        ));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = auth::register_user(&conn, &req.email, &req.password)?;
    Ok(Json(user))
}

pub async fn post_login(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> std::result::Result<Json<LoginResponse>, AppError> {
    if req.email.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest(
            "email and password are required".into(),
        ));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let resp = auth::login_user(&conn, &req.email, &req.password)?;
    Ok(Json(resp))
}

pub async fn list_collections(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Collection>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let cols = db::list_collections(&conn)?;
    Ok(Json(cols))
}

pub async fn create_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateCollectionRequest>,
) -> std::result::Result<Json<Collection>, AppError> {
    let claims = extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &req.name, "admin")?;
    db::insert_collection(&conn, &req.name, &req.schema)?;
    db::create_collection_table(&conn, &req.name, &req.schema)?;
    let c = db::get_collection(&conn, &req.name)?
        .ok_or_else(|| AppError::Internal("collection not found after create".into()))?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_COLLECTION_CREATE,
        webhook::webhook_collection_data(&c),
    );
    Ok(Json(c))
}

pub async fn get_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Json<Collection>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let c =
        db::get_collection(&conn, &name)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(c))
}

pub async fn delete_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "admin")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let c = existing.unwrap();
    webhook::dispatch_event(
        &state,
        webhook::EVENT_COLLECTION_DELETE,
        webhook::webhook_collection_data(&c),
    );
    db::delete_collection(&conn, &name)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<UpdateCollectionRequest>,
) -> std::result::Result<Json<Collection>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "admin")?;
    let c = db::update_collection(&conn, &name, req.schema.as_deref())?;
    Ok(Json(c))
}

pub async fn create_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<CreateRecordRequest>,
) -> std::result::Result<Json<Record>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "editor")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("collection not found".into()));
    }
    let collection = existing.unwrap();
    db::validate_data(&req.data, &collection.schema)?;
    let id = req
        .id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string().replace('-', ""));
    let rec = db::insert_record(&conn, &name, &id, &req.data)?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_CREATE,
        webhook::webhook_record_data(&name, &rec),
    );
    Ok(Json(rec))
}

pub async fn list_records(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<RecordListParams>,
) -> std::result::Result<Json<RecordsPage>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "viewer")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("collection not found".into()));
    }
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let search = params.search.as_deref().unwrap_or("");
    let records = db::list_records(&conn, &name, page, per_page, search)?;
    Ok(Json(records))
}

pub async fn get_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((name, id)): Path<(String, String)>,
) -> std::result::Result<Json<Record>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "viewer")?;
    let rec =
        db::get_record(&conn, &name, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(rec))
}

pub async fn update_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((name, id)): Path<(String, String)>,
    Json(req): Json<UpdateRecordRequest>,
) -> std::result::Result<Json<Record>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "editor")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("collection not found".into()));
    }
    let collection = existing.unwrap();
    db::validate_data(&req.data, &collection.schema)?;
    let rec = db::update_record(&conn, &name, &id, &req.data)?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_UPDATE,
        webhook::webhook_record_data(&name, &rec),
    );
    Ok(Json(rec))
}

pub async fn delete_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((name, id)): Path<(String, String)>,
) -> std::result::Result<StatusCode, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "admin")?;
    let existing = db::get_record(&conn, &name, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let rec = existing.unwrap();
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_DELETE,
        webhook::webhook_record_data(&name, &rec),
    );
    db::delete_record(&conn, &name, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

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

pub async fn list_webhooks(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Webhook>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let hooks = db::list_webhooks(&conn)?;
    Ok(Json(hooks))
}

pub async fn create_webhook(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateWebhookRequest>,
) -> std::result::Result<Json<Webhook>, AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if req.url.is_empty() {
        return Err(AppError::BadRequest("url is required".into()));
    }
    if req.events.is_empty() {
        return Err(AppError::BadRequest(
            "at least one event is required".into(),
        ));
    }
    if let Err(e) = webhook::validate_events(&req.events) {
        return Err(AppError::BadRequest(e));
    }
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let h = db::insert_webhook(&conn, &id, &req.name, &req.url, &req.events, &req.secret)?;
    Ok(Json(h))
}

pub async fn get_webhook(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Webhook>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let h = db::get_webhook(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(h))
}

pub async fn update_webhook(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWebhookRequest>,
) -> std::result::Result<Json<Webhook>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing =
        db::get_webhook(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let url = req.url.unwrap_or(existing.url);
    let events = req.events.unwrap_or(existing.events);
    let secret = req.secret.unwrap_or(existing.secret);
    let is_active = req.is_active.unwrap_or(existing.is_active);

    if let Err(e) = webhook::validate_events(&events) {
        return Err(AppError::BadRequest(e));
    }

    let h = db::update_webhook(&conn, &id, &name, &url, &events, &secret, is_active)?;
    Ok(Json(h))
}

pub async fn delete_webhook(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_webhook(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    db::delete_webhook(&conn, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_webhook_logs(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<WebhookLog>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_webhook(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let logs = db::list_webhook_logs(&conn, &id)?;
    Ok(Json(logs))
}

// --- Secrets ---

#[derive(Serialize)]
pub struct SecretListItem {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn list_secrets(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<SecretListItem>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let secrets = db::list_secrets(&conn)?;
    let items: Vec<SecretListItem> = secrets
        .into_iter()
        .map(|s| SecretListItem {
            id: s.id,
            name: s.name,
            scope: s.scope,
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
        .collect();
    Ok(Json(items))
}

pub async fn create_secret(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateSecretRequest>,
) -> std::result::Result<(StatusCode, Json<Secret>), AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let scope = if req.scope.is_empty() {
        "general".to_string()
    } else {
        req.scope
    };
    let encrypted =
        secrets::encrypt_secret(&state.secrets_key, &req.value).map_err(AppError::Internal)?;
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let secret = db::insert_secret(&conn, &id, &req.name, &encrypted, &scope)?;
    webhook::dispatch_event(
        &state,
        secrets::EVENT_SECRET_CREATE,
        secrets::webhook_secret_data(&secret),
    );
    Ok((StatusCode::CREATED, Json(secret)))
}

pub async fn get_secret(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Secret>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut secret =
        db::get_secret(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    secret.value =
        secrets::decrypt_secret(&state.secrets_key, &secret.value).map_err(AppError::Internal)?;
    Ok(Json(secret))
}

pub async fn update_secret(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSecretRequest>,
) -> std::result::Result<Json<Secret>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing =
        db::get_secret(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let scope = req.scope.unwrap_or(existing.scope);
    let plaintext = match req.value {
        Some(v) if !v.is_empty() => v,
        _ => secrets::decrypt_secret(&state.secrets_key, &existing.value).unwrap_or_default(),
    };
    let encrypted =
        secrets::encrypt_secret(&state.secrets_key, &plaintext).map_err(AppError::Internal)?;
    let secret = db::update_secret(&conn, &id, &name, &encrypted, &scope)?;
    webhook::dispatch_event(
        &state,
        secrets::EVENT_SECRET_UPDATE,
        secrets::webhook_secret_data(&secret),
    );
    Ok(Json(secret))
}

pub async fn delete_secret(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_secret(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    db::delete_secret(&conn, &id)?;
    webhook::dispatch_event(
        &state,
        secrets::EVENT_SECRET_DELETE,
        secrets::webhook_secret_data(&existing.unwrap()),
    );
    Ok(StatusCode::NO_CONTENT)
}

// --- WebSocket Handlers ---

pub async fn list_ws_connections(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<WSConnection>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let connections = db::list_ws_connections(&conn)?;
    Ok(Json(connections))
}

pub async fn get_ws_connection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<WSConnection>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let connection =
        db::get_ws_connection(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(connection))
}

pub async fn delete_ws_connection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let _ = websocket::close_client(&state.ws_hub, &id).await;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_ws_connection(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    db::delete_ws_connection(&conn, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn broadcast_ws_message(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<WSSendRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let content = req.content;

    let ids: Vec<String> = {
        let hub = state.ws_hub.read().await;
        hub.clients.keys().cloned().collect()
    };

    for id in &ids {
        let sender = {
            let hub = state.ws_hub.read().await;
            hub.clients.get(id).map(|c| c.sender.clone())
        };
        if let Some(sender) = sender {
            let msg = Message::Text(content.clone().into());
            sender.send(msg).ok();
        }
        if let Ok(conn) = state.db.get().await {
            db::insert_ws_message(&conn, id, "sent", &content).ok();
        }
    }

    Ok(Json(serde_json::json!({"sent": ids.len()})))
}

pub async fn send_ws_message(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<WSSendRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let content = req.content;

    let sent =
        websocket::send_to_client(&state.ws_hub, &id, Message::Text(content.clone().into())).await;
    if let Ok(conn) = state.db.get().await {
        db::insert_ws_message(&conn, &id, "sent", &content).ok();
    }

    if sent {
        Ok(Json(serde_json::json!({"sent": true})))
    } else {
        Err(AppError::NotFound("connection not found".into()))
    }
}

pub async fn list_ws_messages(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<WSMessage>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_ws_connection(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let messages = db::list_ws_messages(&conn, &id)?;
    Ok(Json(messages))
}

pub async fn list_all_ws_messages(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<WSMessage>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let messages = db::list_all_ws_messages(&conn)?;
    Ok(Json(messages))
}

// --- Cache ---

pub async fn list_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<CacheEntry>>, AppError> {
    extract_claims(&headers)?;
    Ok(Json(state.cache.list()))
}

pub async fn set_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<SetCacheRequest>,
) -> std::result::Result<Json<CacheEntry>, AppError> {
    extract_claims(&headers)?;
    if req.key.is_empty() || req.value.is_empty() {
        return Err(AppError::BadRequest("key and value are required".into()));
    }
    let ttl = req.ttl.unwrap_or(0);
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let entry = state.cache.set(&conn, &req.key, &req.value, ttl)?;
    Ok(Json(entry))
}

pub async fn get_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
) -> std::result::Result<Json<CacheEntry>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let entry = state
        .cache
        .get(&conn, &key)?
        .ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(entry))
}

pub async fn delete_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let ok = state.cache.delete(&conn, &key)?;
    if !ok {
        return Err(AppError::NotFound("not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn flush_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    state.cache.flush(&conn)?;
    Ok(Json(serde_json::json!({"status": "flushed"})))
}

pub async fn get_cache_stats(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    Ok(Json(state.cache.stats()))
}

pub async fn handle_notifications_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Notification>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n = db::list_notifications(&conn)?;
    Ok(Json(n))
}

pub async fn handle_notifications_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateNotificationRequest>,
) -> std::result::Result<Json<Notification>, AppError> {
    extract_claims(&headers)?;

    if req.title.is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let ntype = if req.ntype.is_empty() {
        "info".to_string()
    } else {
        req.ntype.clone()
    };
    if !notification::validate_notification_type(&ntype) {
        return Err(AppError::BadRequest(
            "type must be info, success, warning, or error".into(),
        ));
    }

    let id = Uuid::new_v4().to_string().replace('-', "");
    let notification_entry = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_notification(&conn, &id, &req.title, &req.body, &ntype)?
    };

    notification::broadcast_notification(&state, &notification_entry).await;

    webhook::dispatch_event(
        &state,
        webhook::EVENT_NOTIFICATION_CREATE,
        serde_json::to_value(&notification_entry).unwrap_or_default(),
    );

    Ok(Json(notification_entry))
}

pub async fn handle_notifications_get(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Notification>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n =
        db::get_notification(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(n))
}

pub async fn handle_notifications_mark_read(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Notification>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n = db::update_notification_read(&conn, &id)?;
    Ok(Json(n))
}

pub async fn handle_notifications_delete(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let ok = db::delete_notification(&conn, &id)?;
    if !ok {
        return Err(AppError::NotFound("not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_notifications_clear(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::clear_notifications(&conn)?;
    Ok(Json(serde_json::json!({"status": "cleared"})))
}

pub async fn handle_logs_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<LogEntry>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let logs = db::list_logs(&conn)?;
    Ok(Json(logs))
}

pub async fn handle_logs_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateLogRequest>,
) -> std::result::Result<Json<LogEntry>, AppError> {
    extract_claims(&headers)?;

    if req.message.is_empty() {
        return Err(AppError::BadRequest("message is required".into()));
    }
    let level = if req.level.is_empty() {
        "info".to_string()
    } else {
        req.level.clone()
    };
    if !crate::log::validate_log_level(&level) {
        return Err(AppError::BadRequest(
            "level must be debug, info, warn, or error".into(),
        ));
    }
    let meta = if req.meta.is_empty() {
        "{}".to_string()
    } else {
        req.meta.clone()
    };

    let id = Uuid::new_v4().to_string().replace('-', "");
    let entry = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_log(&conn, &id, &level, &req.message, &meta)?
    };

    crate::log::broadcast_log(&state, &entry).await;

    webhook::dispatch_event(
        &state,
        webhook::EVENT_LOG_CREATE,
        serde_json::to_value(&entry).unwrap_or_default(),
    );

    Ok(Json(entry))
}

pub async fn handle_logs_clear(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::clear_logs(&conn)?;
    Ok(Json(serde_json::json!({"status": "cleared"})))
}

// PubSub handlers

pub async fn handle_pubsub_topics_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<PubSubTopic>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let topics = db::list_pubsub_topics(&conn)?;
    Ok(Json(topics))
}

pub async fn handle_pubsub_topics_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreatePubSubTopicRequest>,
) -> std::result::Result<(StatusCode, Json<PubSubTopic>), AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let existing = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_pubsub_topic_by_name(&conn, &req.name)?
    };
    if existing.is_some() {
        return Err(AppError::Conflict("topic already exists".into()));
    }
    let id = Uuid::new_v4().to_string().replace('-', "");
    let topic = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_pubsub_topic(&conn, &id, &req.name)?
    };
    webhook::dispatch_event(
        &state,
        webhook::EVENT_PUBSUB_TOPIC_CREATE,
        serde_json::json!({"topic": &topic}),
    );
    Ok((StatusCode::CREATED, Json(topic)))
}

pub async fn handle_pubsub_topics_get(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Json<PubSubTopic>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let topic = db::get_pubsub_topic_by_name(&conn, &name)?
        .ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(topic))
}

pub async fn handle_pubsub_topics_delete(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let existing = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_pubsub_topic_by_name(&conn, &name)?
    };
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::delete_pubsub_topic(&conn, &name)?;
    }
    if let Some(topic) = existing {
        webhook::dispatch_event(
            &state,
            webhook::EVENT_PUBSUB_TOPIC_DELETE,
            serde_json::json!({"topic": &topic}),
        );
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_pubsub_messages_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Json<Vec<PubSubMessage>>, AppError> {
    extract_claims(&headers)?;
    let (topic_id,) = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let topic = db::get_pubsub_topic_by_name(&conn, &name)?
            .ok_or_else(|| AppError::NotFound("topic not found".into()))?;
        (topic.id,)
    };
    let msg_list = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::list_pubsub_messages(&conn, &topic_id)?
    };
    Ok(Json(msg_list))
}

pub async fn handle_pubsub_messages_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<CreatePubSubMessageRequest>,
) -> std::result::Result<(StatusCode, Json<PubSubMessage>), AppError> {
    extract_claims(&headers)?;
    if req.body.is_empty() {
        return Err(AppError::BadRequest("body is required".into()));
    }
    let topic = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_pubsub_topic_by_name(&conn, &name)?
            .ok_or_else(|| AppError::NotFound("topic not found".into()))?
    };
    let id = Uuid::new_v4().to_string().replace('-', "");
    let msg = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_pubsub_message(&conn, &id, &topic.id, &req.body)?
    };
    pubsub::broadcast_pubsub(&state, &msg).await;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_PUBSUB_MESSAGE_CREATE,
        serde_json::json!({"message": &msg}),
    );
    Ok((StatusCode::CREATED, Json(msg)))
}

// --- Cron Jobs ---

pub async fn list_cron_jobs(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<CronJob>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let jobs = db::list_cron_jobs(&conn)?;
    Ok(Json(jobs))
}

pub async fn create_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateCronJobRequest>,
) -> std::result::Result<(StatusCode, Json<CronJob>), AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if req.schedule.is_empty() {
        return Err(AppError::BadRequest("schedule is required".into()));
    }
    if req.command.is_empty() {
        return Err(AppError::BadRequest("command is required".into()));
    }
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let method = if req.method.is_empty() {
        "GET".to_string()
    } else {
        req.method
    };
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let job = db::insert_cron_job(
        &conn,
        &id,
        &req.name,
        &req.schedule,
        &req.command,
        &method,
        &req.headers,
        &req.body,
    )?;
    webhook::dispatch_event(
        &state,
        crate::cronjobs::EVENT_CRONJOB_CREATE,
        crate::cronjobs::webhook_cronjob_data(&job),
    );
    Ok((StatusCode::CREATED, Json(job)))
}

pub async fn get_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<CronJob>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let job =
        db::get_cron_job(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(job))
}

pub async fn update_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCronJobRequest>,
) -> std::result::Result<Json<CronJob>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing =
        db::get_cron_job(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let schedule = req.schedule.unwrap_or(existing.schedule);
    let command = req.command.unwrap_or(existing.command);
    let method = req.method.unwrap_or(existing.method);
    let headers = req.headers.unwrap_or(existing.headers);
    let body = req.body.unwrap_or(existing.body);
    let is_active = req.is_active.unwrap_or(existing.is_active);

    let job = db::update_cron_job(
        &conn, &id, &name, &schedule, &command, &method, &headers, &body, is_active,
    )?;
    webhook::dispatch_event(
        &state,
        crate::cronjobs::EVENT_CRONJOB_UPDATE,
        crate::cronjobs::webhook_cronjob_data(&job),
    );
    Ok(Json(job))
}

pub async fn delete_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_cron_job(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let job = existing.unwrap();
    webhook::dispatch_event(
        &state,
        crate::cronjobs::EVENT_CRONJOB_DELETE,
        crate::cronjobs::webhook_cronjob_data(&job),
    );
    db::delete_cron_job(&conn, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn run_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let job = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_cron_job(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?
    };
    crate::cronjobs::execute_cron_job(state, job).await;
    Ok(Json(serde_json::json!({"status": "triggered"})))
}

pub async fn list_cron_job_logs(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<CronJobLog>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_cron_job(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let logs = db::list_cron_job_logs(&conn, &id)?;
    Ok(Json(logs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    fn test_token() -> String {
        use chrono::Utc;
        use jsonwebtoken::{EncodingKey, Header, encode};
        let claims = auth::Claims {
            user_id: "test-user-id".into(),
            email: "test@example.com".into(),
            exp: Utc::now().timestamp() as usize + 3600,
            iat: Utc::now().timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev-secret-change-in-production".into())
                    .as_bytes(),
            ),
        )
        .unwrap()
    }

    // --- extract_token ---

    #[test]
    fn test_extract_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer my-token".parse().unwrap());
        assert_eq!(extract_token(&headers).unwrap(), "my-token");
    }

    #[test]
    fn test_extract_token_missing_header() {
        let headers = HeaderMap::new();
        match extract_token(&headers).unwrap_err() {
            AppError::Unauthorized(_) => {}
            other => panic!("expected Unauthorized, got {other:?}"),
        }
    }

    #[test]
    fn test_extract_token_missing_bearer_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic xxx".parse().unwrap());
        match extract_token(&headers).unwrap_err() {
            AppError::Unauthorized(_) => {}
            other => panic!("expected Unauthorized, got {other:?}"),
        }
    }

    #[test]
    fn test_extract_token_invalid_utf8() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            axum::http::HeaderValue::from_bytes(b"Bearer \xff\xff").unwrap(),
        );
        match extract_token(&headers).unwrap_err() {
            AppError::Unauthorized(_) => {}
            other => panic!("expected Unauthorized, got {other:?}"),
        }
    }

    // --- extract_claims ---

    #[test]
    fn test_extract_claims_valid() {
        let token = test_token();
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {token}").parse().unwrap());
        let claims = extract_claims(&headers).unwrap();
        assert_eq!(claims.user_id, "test-user-id");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_extract_claims_invalid_token() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer garbage-token".parse().unwrap());
        match extract_claims(&headers).unwrap_err() {
            AppError::Unauthorized(_) => {}
            other => panic!("expected Unauthorized, got {other:?}"),
        }
    }

    #[test]
    fn test_extract_claims_missing_header() {
        let headers = HeaderMap::new();
        match extract_claims(&headers).unwrap_err() {
            AppError::Unauthorized(_) => {}
            other => panic!("expected Unauthorized, got {other:?}"),
        }
    }

    // --- get_openapi_json ---

    #[tokio::test]
    async fn test_get_openapi_json_returns_json() {
        let app =
            axum::Router::new().route("/api/openapi.json", axum::routing::get(get_openapi_json));
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            ct.contains("application/json"),
            "expected json content type, got {ct}"
        );
    }

    // --- get_swagger_ui ---

    #[tokio::test]
    async fn test_get_swagger_ui_returns_html() {
        let app = axum::Router::new().route("/api/docs", axum::routing::get(get_swagger_ui));
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/docs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            ct.contains("text/html"),
            "expected html content type, got {ct}"
        );
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(
            html.contains("SwaggerUIBundle"),
            "expected SwaggerUIBundle in body"
        );
    }

    // --- get_health ---

    #[tokio::test]
    async fn test_get_health_returns_ok() {
        let app = axum::Router::new().route("/api/health", axum::routing::get(get_health));
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(val["status"], "ok");
    }
}
