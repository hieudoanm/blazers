//! Client-frame handling: collection subscribe/unsubscribe (with optional
//! `filter`) and presence channel join/leave/state.

use std::sync::Arc;

use axum::extract::ws::Message;
use serde_json::Value;
use tokio::sync::RwLock;

use super::{SUB_ALL, WSHub};

/// Handles `{"action":"subscribe"|"unsubscribe","collection":"name","filter":"..."}`,
/// `{"action":"presence_join"|"presence_leave"|"presence","channel":"c","state":{...}}`.
pub fn handle_client_frame(id: &str, text: &str, hub: &RwLock<WSHub>) {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return;
    };
    let Some(action) = v.get("action").and_then(|a| a.as_str()) else {
        return;
    };
    match action {
        "subscribe" | "unsubscribe" => {
            let collection = v
                .get("collection")
                .and_then(|c| c.as_str())
                .unwrap_or(SUB_ALL)
                .to_string();
            let hub = hub.blocking_read();
            if let Some(client) = hub.clients.get(id) {
                let mut subs = client.subs.lock().unwrap_or_else(|e| e.into_inner());
                if action == "subscribe" {
                    let filter = v
                        .get("filter")
                        .and_then(|f| f.as_str())
                        .map(str::trim)
                        .filter(|f| !f.is_empty())
                        .map(str::to_string);
                    // Reject syntactically invalid filters up front.
                    if filter
                        .as_deref()
                        .is_some_and(|f| blazers_core::filter::compile(f).is_err())
                    {
                        return;
                    }
                    subs.insert(collection, filter);
                } else {
                    subs.remove(&collection);
                }
            }
        }
        "presence_join" | "presence_leave" | "presence" => {
            let Some(channel) = v.get("channel").and_then(|c| c.as_str()) else {
                return;
            };
            presence_action(id, action, channel, v.get("state").cloned(), hub);
        }
        _ => {}
    }
}

fn presence_action(
    id: &str,
    action: &str,
    channel: &str,
    state: Option<Value>,
    hub: &RwLock<WSHub>,
) {
    let mut hub = hub.blocking_write();
    let event_action = match action {
        "presence_join" => {
            hub.presence
                .entry(channel.to_string())
                .or_default()
                .insert(id.to_string(), state.clone().unwrap_or(Value::Null));
            "join"
        }
        "presence_leave" => {
            if let Some(members) = hub.presence.get_mut(channel) {
                members.remove(id);
                if members.is_empty() {
                    hub.presence.remove(channel);
                }
            }
            "leave"
        }
        _ => {
            // "presence" state update — upserts membership (auto-join).
            hub.presence
                .entry(channel.to_string())
                .or_default()
                .insert(id.to_string(), state.clone().unwrap_or(Value::Null));
            "state"
        }
    };
    // Broadcast to all channel members.
    let members: Vec<String> = hub
        .presence
        .get(channel)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let event = serde_json::json!({
        "type": "presence",
        "action": event_action,
        "channel": channel,
        "client": id,
        "state": state,
    });
    let msg = Message::Text(event.to_string().into());
    for client_id in members {
        if let Some(client) = hub.clients.get(&client_id) {
            let _ = client.sender.send(msg.clone());
        }
    }
}

/// Removes a disconnected client from every presence channel and broadcasts
/// `leave` events. Called once when the socket closes.
pub async fn presence_leave_all(hub: &Arc<RwLock<WSHub>>, id: &str) {
    let channels: Vec<String> = {
        let mut hub = hub.write().await;
        let mut out = Vec::new();
        for (channel, members) in hub.presence.iter_mut() {
            if members.remove(id).is_some() {
                out.push(channel.clone());
            }
        }
        hub.presence.retain(|_, m| !m.is_empty());
        out
    };
    for channel in channels {
        let event = serde_json::json!({
            "type": "presence",
            "action": "leave",
            "channel": channel,
            "client": id,
        });
        let msg = Message::Text(event.to_string().into());
        let hub = hub.read().await;
        let members: Vec<String> = hub
            .presence
            .get(&channel)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        for client_id in members {
            if let Some(client) = hub.clients.get(&client_id) {
                let _ = client.sender.send(msg.clone());
            }
        }
    }
}
