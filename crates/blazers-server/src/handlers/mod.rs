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

pub mod collections;
pub use collections::*;
pub mod records;
pub use records::*;
pub mod buckets;
pub use buckets::*;
pub mod files;
pub use files::*;
pub mod webhooks;
pub use webhooks::*;
pub mod secrets_admin;
pub use secrets_admin::*;
pub mod ws;
pub use ws::*;
pub mod cache;
pub use cache::*;
pub mod cache_ops;
pub use cache_ops::*;
pub mod notifications;
pub use notifications::*;
pub mod logs;
pub use logs::*;
pub mod pubsub_admin;
pub use pubsub_admin::*;
pub mod cron;
pub use cron::*;
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

#[cfg(test)]
mod tests;
