use super::*;
use axum::routing::any;
use deadpool::managed::Pool;
use std::path::PathBuf;
use tokio::net::TcpListener;

fn sample_cronjob() -> CronJob {
    CronJob {
        id: "job-123".into(),
        name: "Test Job".into(),
        schedule: "*/5 * * * * *".into(),
        command: "https://example.com/hook".into(),
        method: "POST".into(),
        headers: r#"[["Authorization", "Bearer token"]]"#.into(),
        body: r#"{"key": "value"}"#.into(),
        is_active: true,
        last_run_at: "".into(),
        last_run_status: "".into(),
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2025-01-01T00:00:00Z".into(),
    }
}

fn create_test_env() -> (Arc<AppState>, PathBuf) {
    let tmp_dir = std::env::temp_dir().join(format!("blazers-test-cron-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let db_path = tmp_dir.join("test.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    db::migrate_db(&conn).unwrap();
    drop(conn);
    let pool: Pool<db::ConnectionManager> =
        deadpool::managed::Pool::builder(db::ConnectionManager { path: db_path })
            .build()
            .unwrap();
    let state = Arc::new(AppState {
        db: pool,
        storage_dir: tmp_dir.clone(),
        http_client: reqwest::Client::new(),
        secrets_key: vec![1, 2, 3, 4],
        ws_hub: crate::websocket::new_ws_hub(),
        cache: Arc::new(blazers_kv::cache::CacheStore::new()),
        sse_hub: crate::notification::new_sse_hub(),
        log_hub: crate::log::new_log_hub(),
        pubsub_hub: crate::pubsub::new_pubsub_hub(),
        rate_limiter: Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
    });
    (state, tmp_dir)
}

async fn test_server(status: u16, body: &'static str) -> std::net::SocketAddr {
    let app = axum::Router::new().route(
        "/hook",
        any(move || async move { (axum::http::StatusCode::from_u16(status).unwrap(), body) }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    addr
}

fn make_job(id: &str, method: &str, command: &str, headers: &str, body: &str) -> CronJob {
    CronJob {
        id: id.into(),
        name: format!("{method}-test"),
        schedule: "* * * * * *".into(),
        command: command.into(),
        method: method.into(),
        headers: headers.into(),
        body: body.into(),
        is_active: true,
        last_run_at: String::new(),
        last_run_status: String::new(),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

mod tests_1;
mod tests_2;
mod tests_3;
