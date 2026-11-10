use super::*;
use std::path::PathBuf;

async fn make_rate_limit_app(
    capacity: f64,
    refill_rate: f64,
) -> (
    axum::Router,
    std::sync::Arc<crate::handlers::AppState>,
    PathBuf,
) {
    use axum::{Router, middleware, routing::get};
    use blazers_core::db::ConnectionManager;
    use deadpool::managed::Pool;

    let tmp_dir = std::env::temp_dir().join(format!("blazers-rate-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let db_path = tmp_dir.join("test.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    blazers_core::db::migrate_db(&conn).unwrap();
    drop(conn);
    let pool: Pool<ConnectionManager> = Pool::builder(ConnectionManager { path: db_path })
        .build()
        .unwrap();
    let rate_limiter = std::sync::Arc::new(RateLimiter::new(capacity, refill_rate));
    let state = std::sync::Arc::new(crate::handlers::AppState {
        db: pool,
        storage_dir: tmp_dir.clone(),
        http_client: reqwest::Client::new(),
        secrets_key: vec![1, 2, 3, 4],
        ws_hub: crate::websocket::new_ws_hub(),
        cache: std::sync::Arc::new(crate::cache::CacheStore::new()),
        sse_hub: crate::notification::new_sse_hub(),
        log_hub: crate::log::new_log_hub(),
        pubsub_hub: crate::pubsub::new_pubsub_hub(),
        rate_limiter,
    });

    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .with_state(state.clone());

    (app, state, tmp_dir)
}

mod tests_1;
mod tests_2;
