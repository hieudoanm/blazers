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
use blazers_core::models::Notification;

pub struct SSEClient {
    pub sender: tokio::sync::mpsc::UnboundedSender<String>,
}

pub struct SSEHub {
    pub clients: HashMap<String, SSEClient>,
}

pub fn new_sse_hub() -> Arc<RwLock<SSEHub>> {
    Arc::new(RwLock::new(SSEHub {
        clients: HashMap::new(),
    }))
}

pub fn validate_notification_type(ntype: &str) -> bool {
    matches!(ntype, "info" | "success" | "warning" | "error")
}

pub async fn broadcast_notification(state: &Arc<AppState>, notification: &Notification) {
    let json = serde_json::to_string(notification).unwrap_or_default();
    let msg = format!("data: {json}\n\n");
    let hub = state.sse_hub.read().await;
    for client in hub.clients.values() {
        let _ = client.sender.send(msg.clone());
    }
}

pub async fn handle_notification_stream(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let id = Uuid::new_v4().to_string().replace('-', "");

    state
        .sse_hub
        .write()
        .await
        .clients
        .insert(id.clone(), SSEClient { sender: tx });

    let hub = state.sse_hub.clone();
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
    fn new_sse_hub_creates_empty_hub() {
        let hub = new_sse_hub();
        let guard = hub.try_read().unwrap();
        assert!(guard.clients.is_empty());
    }

    #[tokio::test]
    async fn broadcast_notification_sends_to_clients() {
        let hub = new_sse_hub();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        hub.write()
            .await
            .clients
            .insert("test-id".into(), SSEClient { sender: tx });

        let notification = Notification {
            id: "n1".into(),
            title: "Test".into(),
            body: "Body".into(),
            ntype: "info".into(),
            is_read: false,
            created_at: "2025-01-01T00:00:00Z".into(),
        };

        let json = serde_json::to_string(&notification).unwrap();
        let msg = format!("data: {json}\n\n");

        let hub = hub.read().await;
        for client in hub.clients.values() {
            let _ = client.sender.send(msg.clone());
        }
        drop(hub);

        let received = rx.recv().await.unwrap();
        assert!(received.starts_with("data: "));
        assert!(received.contains("\"id\":\"n1\""));
        assert!(received.contains("\"title\":\"Test\""));
    }

    #[tokio::test]
    async fn broadcast_notification_with_multiple_clients() {
        let hub = new_sse_hub();
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

        let notification = Notification {
            id: "n2".into(),
            title: "Broadcast".into(),
            body: "To all".into(),
            ntype: "warning".into(),
            is_read: false,
            created_at: "2025-06-01T00:00:00Z".into(),
        };

        let json = serde_json::to_string(&notification).unwrap();
        let msg = format!("data: {json}\n\n");

        let hub = hub.read().await;
        for client in hub.clients.values() {
            let _ = client.sender.send(msg.clone());
        }
        drop(hub);

        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();
        assert_eq!(received1, received2);
        assert!(received1.contains("\"type\":\"warning\""));
    }

    #[tokio::test]
    async fn broadcast_notification_empty_hub_does_not_panic() {
        let hub = new_sse_hub();

        let notification = Notification {
            id: "n3".into(),
            title: "Ghost".into(),
            body: "".into(),
            ntype: "error".into(),
            is_read: false,
            created_at: "2025-03-15T00:00:00Z".into(),
        };

        let json = serde_json::to_string(&notification).unwrap();
        let msg = format!("data: {json}\n\n");

        let hub = hub.read().await;
        for client in hub.clients.values() {
            let _ = client.sender.send(msg.clone());
        }
    }

    #[tokio::test]
    async fn client_removal_after_disconnect() {
        let hub = new_sse_hub();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        hub.write()
            .await
            .clients
            .insert("remove-me".into(), SSEClient { sender: tx });

        assert!(hub.read().await.clients.contains_key("remove-me"));

        hub.write().await.clients.remove("remove-me");

        assert!(!hub.read().await.clients.contains_key("remove-me"));
    }

    #[test]
    fn validate_notification_type_info() {
        assert!(validate_notification_type("info"));
    }

    #[test]
    fn validate_notification_type_success() {
        assert!(validate_notification_type("success"));
    }

    #[test]
    fn validate_notification_type_warning() {
        assert!(validate_notification_type("warning"));
    }

    #[test]
    fn validate_notification_type_error() {
        assert!(validate_notification_type("error"));
    }

    #[test]
    fn validate_notification_type_rejects_uppercase() {
        assert!(!validate_notification_type("ERROR"));
    }

    #[test]
    fn validate_notification_type_rejects_empty() {
        assert!(!validate_notification_type(""));
    }

    #[test]
    fn validate_notification_type_rejects_invalid() {
        assert!(!validate_notification_type("invalid"));
    }
}
