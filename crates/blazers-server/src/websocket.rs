use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::handlers::AppState;
use blazers_core::db;

/// All-collections wildcard subscription topic.
pub const SUB_ALL: &str = "*";

/// collection name → optional filter expression (`None` = receive all).
pub type Subs = Mutex<HashMap<String, Option<String>>>;

pub struct WSConsumer {
    pub sender: mpsc::UnboundedSender<Message>,
    pub subs: Subs,
}

pub struct WSHub {
    pub clients: HashMap<String, WSConsumer>,
    /// presence channel → client id → arbitrary state JSON.
    pub presence: HashMap<String, HashMap<String, Value>>,
}

pub fn new_ws_hub() -> Arc<RwLock<WSHub>> {
    Arc::new(RwLock::new(WSHub {
        clients: HashMap::new(),
        presence: HashMap::new(),
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

    let consumer = WSConsumer {
        sender: msg_sender,
        subs: Mutex::new(HashMap::new()),
    };

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
                        let hub = state.ws_hub.clone();
                        let id2 = id.clone();
                        let text2 = text.to_string();
                        tokio::task::spawn_blocking(move || {
                            frames::handle_client_frame(&id2, &text2, &hub);
                        });
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
    frames::presence_leave_all(&state.ws_hub, &id).await;
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

mod broadcast;
pub use broadcast::*;
mod frames;

#[cfg(test)]
mod tests;
