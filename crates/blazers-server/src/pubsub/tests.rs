use super::*;
use std::sync::Arc;

use crate::handlers::AppState;
use blazers_core::models::PubSubMessage;

#[test]
fn new_pubsub_hub_creates_non_null_hub() {
    let hub = new_pubsub_hub();
    assert!(hub.try_read().is_ok());
}

#[test]
fn new_pubsub_hub_starts_with_empty_clients() {
    let hub = new_pubsub_hub();
    let guard = hub.try_read().unwrap();
    assert!(guard.clients.is_empty());
}

#[test]
fn new_pubsub_hub_read_write_roundtrip() {
    let hub = new_pubsub_hub();
    {
        let mut guard = hub.try_write().unwrap();
        guard.clients.insert(
            "test-id".into(),
            SSEClient {
                sender: tokio::sync::mpsc::unbounded_channel().0,
            },
        );
    }
    let guard = hub.try_read().unwrap();
    assert_eq!(guard.clients.len(), 1);
    assert!(guard.clients.contains_key("test-id"));
}

fn test_app_state(hub: Arc<RwLock<SSEHub>>) -> Arc<AppState> {
    let tmp_dir = std::env::temp_dir().join("blazers-test-pubsub");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let pool = deadpool::managed::Pool::builder(blazers_core::db::ConnectionManager {
        path: tmp_dir.join("test.db"),
    })
    .build()
    .unwrap();
    Arc::new(AppState {
        db: pool,
        storage_dir: tmp_dir,
        http_client: reqwest::Client::new(),
        secrets_key: vec![1, 2, 3, 4],
        ws_hub: crate::websocket::new_ws_hub(),
        cache: Arc::new(blazers_kv::cache::CacheStore::new()),
        sse_hub: crate::notification::new_sse_hub(),
        log_hub: crate::log::new_log_hub(),
        pubsub_hub: hub,
        rate_limiter: Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
    })
}

#[tokio::test]
async fn broadcast_pubsub_sends_to_all_clients() {
    let hub = new_pubsub_hub();
    let msg = PubSubMessage {
        id: "msg_1".into(),
        topic_id: "topic_1".into(),
        body: "test body".into(),
        created_at: "2025-01-01T00:00:00Z".into(),
    };

    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel::<String>();

    {
        let mut guard = hub.write().await;
        guard.clients.insert("c1".into(), SSEClient { sender: tx1 });
        guard.clients.insert("c2".into(), SSEClient { sender: tx2 });
    }

    let state = test_app_state(hub);
    broadcast_pubsub(&state, &msg).await;

    let json = serde_json::to_string(&msg).unwrap();
    let expected = format!("data: {json}\n\n");

    let received1 = rx1.try_recv().expect("c1 should receive");
    let received2 = rx2.try_recv().expect("c2 should receive");
    assert_eq!(received1, expected);
    assert_eq!(received2, expected);
}

#[tokio::test]
async fn broadcast_pubsub_skips_disconnected_clients() {
    let hub = new_pubsub_hub();
    let msg = PubSubMessage {
        id: "msg_2".into(),
        topic_id: "topic_2".into(),
        body: "disconnect test".into(),
        created_at: "2025-01-01T00:00:00Z".into(),
    };

    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel::<String>();

    {
        let mut guard = hub.write().await;
        guard
            .clients
            .insert("alive".into(), SSEClient { sender: tx1 });
        guard
            .clients
            .insert("dead".into(), SSEClient { sender: tx2 });
    }

    drop(rx2);

    let state = test_app_state(hub);
    broadcast_pubsub(&state, &msg).await;

    let json = serde_json::to_string(&msg).unwrap();
    let expected = format!("data: {json}\n\n");

    let received = rx1.try_recv().expect("alive client should receive");
    assert_eq!(received, expected);
}

#[tokio::test]
async fn broadcast_pubsub_sends_formatted_sse_data() {
    let hub = new_pubsub_hub();
    let msg = PubSubMessage {
        id: "msg_3".into(),
        topic_id: "topic_3".into(),
        body: "sse format".into(),
        created_at: "2025-01-01T00:00:00Z".into(),
    };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    {
        let mut guard = hub.write().await;
        guard.clients.insert("c1".into(), SSEClient { sender: tx });
    }

    let state = test_app_state(hub);
    broadcast_pubsub(&state, &msg).await;

    let received = rx.try_recv().expect("should receive message");
    assert!(received.starts_with("data: "));
    assert!(received.ends_with("\n\n"));
    assert!(received.contains(&msg.id));
    assert!(received.contains(&msg.body));
}

#[tokio::test]
async fn handle_pubsub_stream_adds_client_to_hub() {
    let hub = new_pubsub_hub();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let id = Uuid::new_v4().to_string().replace('-', "");

    {
        let mut guard = hub.write().await;
        guard.clients.insert(id.clone(), SSEClient { sender: tx });
    }

    let guard = hub.read().await;
    assert!(guard.clients.contains_key(&id));
    assert_eq!(guard.clients.len(), 1);
}

#[tokio::test]
async fn handle_pubsub_stream_removes_client_on_cleanup() {
    let hub = new_pubsub_hub();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let id = Uuid::new_v4().to_string().replace('-', "");

    {
        let mut guard = hub.write().await;
        guard.clients.insert(id.clone(), SSEClient { sender: tx });
    }

    {
        let mut guard = hub.write().await;
        guard.clients.remove(&id);
    }

    let guard = hub.read().await;
    assert!(!guard.clients.contains_key(&id));
    assert!(guard.clients.is_empty());
    drop(rx);
}
