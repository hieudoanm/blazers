use super::*;

fn create_perms_table(conn: &rusqlite::Connection) {
    conn.execute_batch(
        "CREATE TABLE _permissions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            collection TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT '',
            UNIQUE(user_id, collection)
        );",
    )
    .unwrap();
}

async fn exhausted_pool_app() -> (axum::Router, std::sync::Arc<crate::handlers::AppState>) {
    use blazers_core::db::ConnectionManager;
    use deadpool::Runtime::Tokio1;
    use deadpool::managed::Pool;

    let pool: Pool<ConnectionManager> = Pool::builder(ConnectionManager {
        path: std::path::PathBuf::from(":memory:"),
    })
    .max_size(1)
    .runtime(Tokio1)
    .create_timeout(Some(std::time::Duration::from_millis(1)))
    .wait_timeout(Some(std::time::Duration::from_millis(1)))
    .build()
    .unwrap();
    let _conn = pool.get().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let state = std::sync::Arc::new(crate::handlers::AppState {
        db: pool,
        storage_dir: std::env::temp_dir(),
        http_client: reqwest::Client::new(),
        secrets_key: vec![1, 2, 3, 4],
        ws_hub: crate::websocket::new_ws_hub(),
        cache: std::sync::Arc::new(crate::cache::CacheStore::new()),
        sse_hub: crate::notification::new_sse_hub(),
        log_hub: crate::log::new_log_hub(),
        pubsub_hub: crate::pubsub::new_pubsub_hub(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
    });
    let app = axum::Router::new()
        .route(
            "/api/permissions",
            axum::routing::get(handle_list_permissions).post(handle_create_permission),
        )
        .route(
            "/api/permissions/{id}",
            axum::routing::delete(handle_delete_permission),
        )
        .with_state(state.clone());
    (app, state)
}

fn test_token() -> String {
    use chrono::Utc;
    use jsonwebtoken::{EncodingKey, Header, encode};
    let claims = crate::auth::Claims {
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

async fn test_rbac_app() -> (axum::Router, std::sync::Arc<crate::handlers::AppState>) {
    use axum::routing::{delete, get};
    use blazers_core::db::ConnectionManager;
    use deadpool::managed::Pool;

    let tmp_dir = std::env::temp_dir().join(format!("blazers-rbac-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).ok();
    let db_path = tmp_dir.join("test.db");
    let pool: Pool<ConnectionManager> = Pool::builder(ConnectionManager { path: db_path })
        .build()
        .unwrap();
    let conn = pool.get().await.unwrap();
    blazers_core::db::migrate_db(&conn).expect("migrate");
    drop(conn);
    let state = std::sync::Arc::new(crate::handlers::AppState {
        db: pool,
        storage_dir: tmp_dir,
        http_client: reqwest::Client::new(),
        secrets_key: vec![1, 2, 3, 4],
        ws_hub: crate::websocket::new_ws_hub(),
        cache: std::sync::Arc::new(crate::cache::CacheStore::new()),
        sse_hub: crate::notification::new_sse_hub(),
        log_hub: crate::log::new_log_hub(),
        pubsub_hub: crate::pubsub::new_pubsub_hub(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
    });
    let app = axum::Router::new()
        .route(
            "/api/permissions",
            get(handle_list_permissions).post(handle_create_permission),
        )
        .route("/api/permissions/{id}", delete(handle_delete_permission))
        .with_state(state.clone());
    (app, state)
}

async fn request_with_auth(
    app: &axum::Router,
    method: axum::http::Method,
    path: &str,
    token: &str,
    body: Option<serde_json::Value>,
) -> (axum::http::StatusCode, serde_json::Value) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let req_builder = Request::builder()
        .method(method)
        .uri(path)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json");
    let b = body
        .map(|v| Body::from(serde_json::to_string(&v).unwrap()))
        .unwrap_or_else(Body::empty);
    let req = req_builder.body(b).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let value: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
    (status, value)
}

mod tests_1;
mod tests_2;
mod tests_3;
mod tests_4;
