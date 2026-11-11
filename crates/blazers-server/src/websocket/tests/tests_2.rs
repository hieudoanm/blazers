use super::*;

#[tokio::test]
async fn handle_socket_stores_received_message_in_db() {
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

    ws_stream
        .send(tokio_tungstenite::tungstenite::Message::Text(
            "hello world".into(),
        ))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let db = state.db.get().await.unwrap();
    let connections = blazers_core::db::list_ws_connections(&db).unwrap();
    assert_eq!(connections.len(), 1);
    let messages = blazers_core::db::list_ws_messages(&db, &connections[0].id).unwrap();
    assert_eq!(messages.len(), 1, "should have one message stored");
    assert_eq!(messages[0].content, "hello world");
    assert_eq!(messages[0].direction, "received");

    ws_stream.close(None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    std::fs::remove_dir_all(&tmp_dir).ok();
}

#[tokio::test]
async fn handle_socket_sends_message_to_client() {
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

    // Send a message to ourselves via the hub using send_to_client
    // First, we need the client ID. Since handle_socket creates it internally,
    // we need to find it from the hub.
    let client_id = {
        let guard = ws_hub.read().await;
        guard.clients.keys().next().cloned().unwrap()
    };

    let msg = Message::Text("from server".into());
    send_to_client(&ws_hub, &client_id, msg).await;

    // Verify the message was received on the WebSocket connection. The server
    // pings on an interval, so skip control frames until the text arrives.
    use tokio::time::timeout;
    use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
    let received = timeout(Duration::from_secs(5), async {
        loop {
            match ws_stream.next().await {
                Some(Ok(TungsteniteMessage::Text(t))) => break t,
                Some(Ok(TungsteniteMessage::Ping(_) | TungsteniteMessage::Pong(_))) => continue,
                Some(Ok(other)) => panic!("expected text message, got: {other:?}"),
                Some(Err(err)) => panic!("websocket error: {err}"),
                None => panic!("websocket closed before message arrived"),
            }
        }
    })
    .await
    .expect("timed out waiting for message");
    assert_eq!(received, "from server");

    ws_stream.close(None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    std::fs::remove_dir_all(&tmp_dir).ok();
}
