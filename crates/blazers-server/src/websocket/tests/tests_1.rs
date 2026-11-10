use super::*;

#[test]
fn new_ws_hub_creates_non_null_hub() {
    let hub = new_ws_hub();
    assert!(hub.try_read().is_ok());
}

#[tokio::test]
async fn ws_hub_run_can_be_cancelled() {
    let hub = new_ws_hub();
    let task = tokio::spawn(ws_hub_run(hub));
    tokio::time::sleep(Duration::from_millis(10)).await;
    task.abort();
    assert!(task.await.is_err());
}

#[tokio::test]
async fn send_to_client_unknown_returns_false() {
    let hub = new_ws_hub();
    let result = send_to_client(&hub, "nonexistent", Message::Text("hi".into())).await;
    assert!(!result);
}

#[tokio::test]
async fn send_to_client_success_returns_true() {
    let hub = new_ws_hub();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = Uuid::new_v4().to_string();

    hub.write()
        .await
        .clients
        .insert(id.clone(), WSConsumer { sender: tx });

    let msg = Message::Text("hello from test".into());
    let result = send_to_client(&hub, &id, msg.clone()).await;
    assert!(result);
    assert_eq!(rx.recv().await, Some(msg));
}

#[tokio::test]
async fn send_to_client_channel_closed_returns_false() {
    let hub = new_ws_hub();
    let (tx, rx) = mpsc::unbounded_channel();
    drop(rx);
    let id = Uuid::new_v4().to_string();

    hub.write()
        .await
        .clients
        .insert(id.clone(), WSConsumer { sender: tx });

    let result = send_to_client(&hub, &id, Message::Text("hello".into())).await;
    assert!(!result);
}

#[tokio::test]
async fn close_client_unknown_returns_false() {
    let hub = new_ws_hub();
    let result = close_client(&hub, "nonexistent").await;
    assert!(!result);
}

#[tokio::test]
async fn close_client_success_returns_true() {
    let hub = new_ws_hub();
    let (tx, _rx) = mpsc::unbounded_channel();
    let id = Uuid::new_v4().to_string();

    hub.write()
        .await
        .clients
        .insert(id.clone(), WSConsumer { sender: tx });

    let result = close_client(&hub, &id).await;
    assert!(result);
    assert!(!hub.read().await.clients.contains_key(&id));
}

#[tokio::test]
async fn handle_ws_upgrade_rejects_non_get_methods() {
    let tmp_dir = std::env::temp_dir().join(format!("ws-test-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let db_path = tmp_dir.join("test.db");
    let pool: deadpool::managed::Pool<db::ConnectionManager> =
        deadpool::managed::Pool::builder(db::ConnectionManager { path: db_path })
            .build()
            .unwrap();
    let conn = pool.get().await.unwrap();
    blazers_core::db::migrate_db(&conn).unwrap();
    drop(conn);
    let key = crate::secrets::get_or_create_secrets_key(&tmp_dir).unwrap();
    let ws_hub = new_ws_hub();
    let cache = Arc::new(crate::cache::CacheStore::new());
    let sse_hub = crate::notification::new_sse_hub();
    let log_hub = crate::log::new_log_hub();
    let pubsub_hub = crate::pubsub::new_pubsub_hub();
    let rate_limiter = Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0));
    let state = Arc::new(AppState {
        db: pool,
        storage_dir: tmp_dir.clone(),
        http_client: reqwest::Client::new(),
        secrets_key: key,
        ws_hub,
        cache,
        sse_hub,
        log_hub,
        pubsub_hub,
        rate_limiter,
    });

    let app = Router::new()
        .route("/ws", get(handle_ws_upgrade))
        .with_state(state);

    let req = axum::http::Request::builder()
        .method("DELETE")
        .uri("/ws")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 405);

    // Clean up temp dir
    std::fs::remove_dir_all(&tmp_dir).ok();
}

#[tokio::test]
async fn handle_socket_connects_and_stores_in_db() {
    let tmp_dir = std::env::temp_dir().join(format!("ws-test-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let db_path = tmp_dir.join("test.db");
    let pool: deadpool::managed::Pool<db::ConnectionManager> =
        deadpool::managed::Pool::builder(db::ConnectionManager { path: db_path })
            .build()
            .unwrap();
    let conn = pool.get().await.unwrap();
    blazers_core::db::migrate_db(&conn).unwrap();
    drop(conn);

    let key = crate::secrets::get_or_create_secrets_key(&tmp_dir).unwrap();
    let ws_hub = new_ws_hub();
    let cache = Arc::new(crate::cache::CacheStore::new());
    let sse_hub = crate::notification::new_sse_hub();
    let log_hub = crate::log::new_log_hub();
    let pubsub_hub = crate::pubsub::new_pubsub_hub();
    let rate_limiter = Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0));
    let state = Arc::new(AppState {
        db: pool,
        storage_dir: tmp_dir.clone(),
        http_client: reqwest::Client::new(),
        secrets_key: key,
        ws_hub: ws_hub.clone(),
        cache,
        sse_hub,
        log_hub,
        pubsub_hub,
        rate_limiter,
    });

    let app = Router::new()
        .route("/ws", get(handle_ws_upgrade))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

    ws_stream.close(None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    let db = state.db.get().await.unwrap();
    let connections = blazers_core::db::list_ws_connections(&db).unwrap();
    assert_eq!(connections.len(), 1, "should have one connection");
    assert!(
        !connections[0].is_active,
        "connection should be inactive after close"
    );

    let guard = ws_hub.read().await;
    assert!(
        guard.clients.is_empty(),
        "client should be removed from hub"
    );

    std::fs::remove_dir_all(&tmp_dir).ok();
}
