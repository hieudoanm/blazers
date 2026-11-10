use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::sse::{Event, Sse};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::handlers::AppState;
use blazers_core::models::LogEntry;

pub struct SSEClient {
    pub sender: tokio::sync::mpsc::UnboundedSender<String>,
}

pub struct SSEHub {
    pub clients: HashMap<String, SSEClient>,
}

pub fn new_log_hub() -> Arc<RwLock<SSEHub>> {
    Arc::new(RwLock::new(SSEHub {
        clients: HashMap::new(),
    }))
}

pub fn validate_log_level(level: &str) -> bool {
    matches!(level, "debug" | "info" | "warn" | "error")
}

pub async fn broadcast_log(state: &Arc<AppState>, entry: &LogEntry) {
    let json = serde_json::to_string(entry).unwrap_or_default();
    let msg = format!("data: {json}\n\n");
    let hub = state.log_hub.read().await;
    for client in hub.clients.values() {
        let _ = client.sender.send(msg.clone());
    }
}

pub async fn handle_log_stream(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let id = Uuid::new_v4().to_string().replace('-', "");

    state
        .log_hub
        .write()
        .await
        .clients
        .insert(id.clone(), SSEClient { sender: tx });

    let hub = state.log_hub.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        hub.write().await.clients.remove(&id);
    });

    let stream = UnboundedReceiverStream::new(rx)
        .map(|msg| Ok::<Event, Infallible>(Event::default().data(msg)));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log_hub_creates_non_null_hub() {
        let hub = new_log_hub();
        assert!(hub.try_read().is_ok());
    }

    #[test]
    fn new_log_hub_starts_with_empty_clients() {
        let hub = new_log_hub();
        let guard = hub.try_read().unwrap();
        assert!(guard.clients.is_empty());
    }

    #[test]
    fn new_log_hub_read_write_roundtrip() {
        let hub = new_log_hub();
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

    #[tokio::test]
    async fn hub_broadcast_sends_to_all_clients() {
        let hub = new_log_hub();
        let entry = LogEntry {
            id: "log_1".into(),
            level: "info".into(),
            message: "test message".into(),
            meta: "{}".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
        };

        let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel::<String>();

        {
            let mut guard = hub.write().await;
            guard
                .clients
                .insert("client-1".into(), SSEClient { sender: tx1 });
            guard
                .clients
                .insert("client-2".into(), SSEClient { sender: tx2 });
        }

        let json = serde_json::to_string(&entry).unwrap_or_default();
        let msg = format!("data: {json}\n\n");

        {
            let guard = hub.read().await;
            for client in guard.clients.values() {
                let _ = client.sender.send(msg.clone());
            }
        }

        let received1 = rx1.try_recv().expect("client-1 should receive message");
        let received2 = rx2.try_recv().expect("client-2 should receive message");
        assert_eq!(received1, msg);
        assert_eq!(received2, msg);
    }

    #[tokio::test]
    async fn hub_broadcast_skips_disconnected_clients() {
        let hub = new_log_hub();
        let entry = LogEntry {
            id: "log_2".into(),
            level: "warn".into(),
            message: "disconnect test".into(),
            meta: "{}".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
        };

        let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel::<String>();

        {
            let mut guard = hub.write().await;
            guard
                .clients
                .insert("client-alive".into(), SSEClient { sender: tx1 });
            guard
                .clients
                .insert("client-dead".into(), SSEClient { sender: tx2 });
        }

        drop(rx2);

        let json = serde_json::to_string(&entry).unwrap_or_default();
        let msg = format!("data: {json}\n\n");

        {
            let guard = hub.read().await;
            for client in guard.clients.values() {
                let _ = client.sender.send(msg.clone());
            }
        }

        let received = rx1.try_recv().expect("alive client should receive message");
        assert_eq!(received, msg);
    }

    #[test]
    fn validate_log_level_debug() {
        assert!(validate_log_level("debug"));
    }

    #[test]
    fn validate_log_level_info() {
        assert!(validate_log_level("info"));
    }

    #[test]
    fn validate_log_level_warn() {
        assert!(validate_log_level("warn"));
    }

    #[test]
    fn validate_log_level_error() {
        assert!(validate_log_level("error"));
    }

    #[test]
    fn validate_log_level_rejects_uppercase() {
        assert!(!validate_log_level("INFO"));
    }

    #[test]
    fn validate_log_level_rejects_empty() {
        assert!(!validate_log_level(""));
    }

    #[test]
    fn validate_log_level_rejects_invalid() {
        assert!(!validate_log_level("invalid"));
    }
}
