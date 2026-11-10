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
mod tests;
