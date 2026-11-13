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

    hub.write().await.clients.insert(
        id.clone(),
        WSConsumer {
            sender: tx,
            subs: std::sync::Mutex::new(std::collections::HashMap::new()),
        },
    );

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

    hub.write().await.clients.insert(
        id.clone(),
        WSConsumer {
            sender: tx,
            subs: std::sync::Mutex::new(std::collections::HashMap::new()),
        },
    );

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

    hub.write().await.clients.insert(
        id.clone(),
        WSConsumer {
            sender: tx,
            subs: std::sync::Mutex::new(std::collections::HashMap::new()),
        },
    );

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

#[tokio::test]
async fn broadcast_to_collection_only_reaches_subscribers() {
    let hub = new_ws_hub();
    let (tx_sub, mut rx_sub) = mpsc::unbounded_channel();
    let (tx_other, mut rx_other) = mpsc::unbounded_channel();
    let (tx_all, mut rx_all) = mpsc::unbounded_channel();

    let mut subs_sub = std::collections::HashMap::new();
    subs_sub.insert("posts".to_string(), None);
    let mut subs_all = std::collections::HashMap::new();
    subs_all.insert(SUB_ALL.to_string(), None);

    hub.write().await.clients.insert(
        "sub".into(),
        WSConsumer {
            sender: tx_sub,
            subs: std::sync::Mutex::new(subs_sub),
        },
    );
    hub.write().await.clients.insert(
        "other".into(),
        WSConsumer {
            sender: tx_other,
            subs: std::sync::Mutex::new(std::collections::HashMap::new()),
        },
    );
    hub.write().await.clients.insert(
        "all".into(),
        WSConsumer {
            sender: tx_all,
            subs: std::sync::Mutex::new(subs_all),
        },
    );

    let payload = serde_json::json!({"type": "record", "action": "create", "collection": "posts"});
    let sent = broadcast_to_collection(&hub, "posts", &payload).await;
    assert_eq!(sent, 2);
    assert!(rx_sub.recv().await.is_some());
    assert!(rx_all.recv().await.is_some());
    assert!(rx_other.try_recv().is_err());

    // different collection reaches only the wildcard subscriber
    let sent = broadcast_to_collection(&hub, "comments", &payload).await;
    assert_eq!(sent, 1);
    assert!(rx_all.recv().await.is_some());
}

#[tokio::test]
async fn broadcast_record_wraps_payload() {
    let hub = new_ws_hub();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut subs = std::collections::HashMap::new();
    subs.insert("posts".to_string(), None);
    hub.write().await.clients.insert(
        "c".into(),
        WSConsumer {
            sender: tx,
            subs: std::sync::Mutex::new(subs),
        },
    );
    let rec = serde_json::json!({"id": "r1"});
    broadcast_record(&hub, "posts", "update", &rec).await;
    let msg = rx.recv().await.unwrap();
    let Message::Text(text) = msg else {
        panic!("expected text")
    };
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["type"], "record");
    assert_eq!(v["action"], "update");
    assert_eq!(v["collection"], "posts");
    assert_eq!(v["record"]["id"], "r1");
}

#[tokio::test]
async fn broadcast_record_change_includes_old_record() {
    let hub = new_ws_hub();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut subs = std::collections::HashMap::new();
    subs.insert("posts".to_string(), None);
    hub.write().await.clients.insert(
        "c".into(),
        WSConsumer {
            sender: tx,
            subs: std::sync::Mutex::new(subs),
        },
    );
    let rec = serde_json::json!({"id": "r1", "data": {"v": 2}});
    let old = serde_json::json!({"id": "r1", "data": {"v": 1}});
    broadcast_record_change(&hub, "posts", "update", &rec, Some(&old)).await;
    let msg = rx.recv().await.unwrap();
    let Message::Text(text) = msg else {
        panic!("expected text")
    };
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["oldRecord"]["data"]["v"], 1);
    assert_eq!(v["record"]["data"]["v"], 2);
}
