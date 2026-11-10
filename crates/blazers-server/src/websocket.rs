use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::handlers::AppState;
use blazers_core::db;

pub struct WSConsumer {
    pub sender: mpsc::UnboundedSender<Message>,
}

pub struct WSHub {
    pub clients: HashMap<String, WSConsumer>,
}

pub fn new_ws_hub() -> Arc<RwLock<WSHub>> {
    Arc::new(RwLock::new(WSHub {
        clients: HashMap::new(),
    }))
}

pub async fn ws_hub_run(_hub: Arc<RwLock<WSHub>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}

pub async fn handle_ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_agent))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>, user_agent: String) {
    let id = Uuid::new_v4().to_string().replace('-', "");
    let path = "/ws".to_string();
    let remote_addr = String::new();

    let (msg_sender, mut msg_receiver) = mpsc::unbounded_channel::<Message>();

    let consumer = WSConsumer { sender: msg_sender };

    state
        .ws_hub
        .write()
        .await
        .clients
        .insert(id.clone(), consumer);
    if let Ok(conn) = state.db.get().await {
        db::insert_ws_connection(&conn, &id, &remote_addr, &path, &user_agent).ok();
    }

    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(conn) = state.db.get().await {
                            db::insert_ws_message(&conn, &id, "received", &text).ok();
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            msg = msg_receiver.recv() => {
                match msg {
                    Some(Message::Close(_)) => {
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Some(msg) => {
                        if socket.send(msg).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                if socket.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }

    state.ws_hub.write().await.clients.remove(&id);
    if let Ok(conn) = state.db.get().await {
        db::update_ws_disconnect(&conn, &id).ok();
    }
}

pub async fn send_to_client(hub: &RwLock<WSHub>, id: &str, message: Message) -> bool {
    let hub = hub.read().await;
    if let Some(client) = hub.clients.get(id) {
        client.sender.send(message).is_ok()
    } else {
        false
    }
}

pub async fn close_client(hub: &RwLock<WSHub>, id: &str) -> bool {
    let mut hub = hub.write().await;
    if let Some(consumer) = hub.clients.remove(id) {
        let _ = consumer.sender.send(Message::Close(None));
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::get;
    use futures::{SinkExt, StreamExt};
    use tower::ServiceExt;

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

        // Verify the message was received on the WebSocket connection
        tokio::time::sleep(Duration::from_millis(200)).await;
        use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
        let received = ws_stream
            .next()
            .await
            .expect("should receive a message")
            .expect("message should be ok");
        match received {
            TungsteniteMessage::Text(t) => assert_eq!(t, "from server"),
            other => panic!("expected text message, got: {other:?}"),
        }

        ws_stream.close(None).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;

        std::fs::remove_dir_all(&tmp_dir).ok();
    }
}
