use super::*;
use crate::handlers::AppState;
use crate::{cache, notification, pubsub, rate_limit, secrets, websocket};
use axum::Router;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use blazers_core::db;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

async fn test_app() -> Router {
    test_app_with_state().await.0
}

async fn test_app_with_state() -> (Router, Arc<AppState>) {
    let tmp_dir = std::env::temp_dir().join(format!("blazers-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).ok();
    let db_path = tmp_dir.join("test.db");
    let pool: deadpool::managed::Pool<db::ConnectionManager> =
        deadpool::managed::Pool::builder(db::ConnectionManager { path: db_path })
            .build()
            .unwrap();
    let conn = pool.get().await.unwrap();
    db::migrate_db(&conn).expect("migrate");
    drop(conn);
    let key = secrets::get_or_create_secrets_key(&tmp_dir).unwrap();
    let ws_hub = websocket::new_ws_hub();
    let cache = std::sync::Arc::new(cache::CacheStore::new());
    let sse_hub = notification::new_sse_hub();
    let log_hub = crate::log::new_log_hub();
    let pubsub_hub = pubsub::new_pubsub_hub();
    let rate_limiter = std::sync::Arc::new(rate_limit::RateLimiter::new(200.0, 100.0));
    let state = Arc::new(AppState {
        db: pool,
        storage_dir: tmp_dir,
        http_client: reqwest::Client::new(),
        secrets_key: key,
        ws_hub,
        cache,
        sse_hub,
        log_hub,
        pubsub_hub,
        rate_limiter,
    });
    (app(state.clone()), state)
}

async fn post_json(app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn get_json(app: &Router, path: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut req_builder = Request::builder().method("GET").uri(path);
    if let Some(t) = token {
        req_builder = req_builder.header("Authorization", format!("Bearer {t}"));
    }
    let req = req_builder.body(Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req_builder = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if let Some(t) = token {
        req_builder = req_builder.header("Authorization", format!("Bearer {t}"));
    }
    let b = body
        .map(|v| Body::from(serde_json::to_string(&v).unwrap()))
        .unwrap_or_else(Body::empty);
    let req = req_builder.body(b).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn register_token(app: &Router) -> String {
    let body = serde_json::json!({"email": "admin@test.com", "password": "admin123"});
    let (_, resp) = post_json(app, "/api/auth/register", body).await;
    let email = resp["email"].as_str().unwrap_or("");
    assert!(!email.is_empty(), "register should succeed");

    let body = serde_json::json!({"email": "admin@test.com", "password": "admin123"});
    let (_, resp) = post_json(app, "/api/auth/login", body).await;
    resp["token"].as_str().unwrap_or("").to_string()
}

async fn register_admin(app: &Router, state: &Arc<AppState>) -> String {
    let body = serde_json::json!({"email": "admin@test.com", "password": "admin123"});
    let (_, resp) = post_json(app, "/api/auth/register", body).await;
    assert!(
        !resp["email"].as_str().unwrap_or("").is_empty(),
        "register should succeed"
    );

    let body = serde_json::json!({"email": "admin@test.com", "password": "admin123"});
    let (_, resp) = post_json(app, "/api/auth/login", body).await;
    let token = resp["token"].as_str().unwrap_or("").to_string();
    let user_id = resp["user"]["id"].as_str().unwrap_or("").to_string();

    let conn = state.db.get().await.unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO _permissions (id, user_id, collection, role) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![uuid::Uuid::new_v4().to_string(), user_id, "*", "admin"],
    ).ok();

    token
}

fn multipart_body(filename: &str, content: &[u8]) -> (String, Vec<u8>) {
    let boundary = "----testboundary123";
    let mut buf = Vec::new();
    buf.extend_from_slice(b"--");
    buf.extend_from_slice(boundary.as_bytes());
    buf.extend_from_slice(b"\r\n");
    buf.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
            .as_bytes(),
    );
    buf.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    buf.extend_from_slice(content);
    buf.extend_from_slice(b"\r\n--");
    buf.extend_from_slice(boundary.as_bytes());
    buf.extend_from_slice(b"--\r\n");
    let content_type = format!("multipart/form-data; boundary={boundary}");
    (content_type, buf)
}

async fn upload_file(
    app: &Router,
    bucket: &str,
    filename: &str,
    content: &[u8],
    token: &str,
) -> (StatusCode, Value) {
    let (content_type, body) = multipart_body(filename, content);
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/buckets/{bucket}/files"))
        .header("Content-Type", &content_type)
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
    (status, value)
}

mod tests_1;
mod tests_10;
mod tests_2;
mod tests_3;
mod tests_4;
mod tests_5;
mod tests_6;
mod tests_7;
mod tests_8;
mod tests_9;
