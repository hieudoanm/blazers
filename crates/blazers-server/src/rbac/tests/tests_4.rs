use super::*;

#[tokio::test]
async fn test_handle_permissions_db_error_returns_internal_error() {
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
        cache: std::sync::Arc::new(blazers_kv::cache::CacheStore::new()),
        sse_hub: crate::notification::new_sse_hub(),
        log_hub: crate::log::new_log_hub(),
        pubsub_hub: crate::pubsub::new_pubsub_hub(),
        rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
    });
    let app = axum::Router::new()
        .route(
            "/api/permissions",
            axum::routing::get(handle_list_permissions),
        )
        .with_state(state.clone());
    let token = test_token();
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    let req = Request::builder()
        .uri("/api/permissions")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_handle_create_permission_db_error() {
    let (app, _state) = exhausted_pool_app().await;
    let token = test_token();
    let body = serde_json::json!({"user_id": "u1", "collection": "c1", "role": "viewer"});
    let (status, _) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_handle_delete_permission_db_error() {
    let (app, _state) = exhausted_pool_app().await;
    let token = test_token();
    let (status, _) = request_with_auth(
        &app,
        axum::http::Method::DELETE,
        "/api/permissions/some-id",
        &token,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}
