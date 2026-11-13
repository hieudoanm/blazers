//! Collection-scoped broadcast helpers for the WS hub.

use axum::extract::ws::Message;
use serde_json::Value;
use tokio::sync::RwLock;

use super::{SUB_ALL, WSHub};

/// Sends `payload` (a JSON value) to every client subscribed to `collection`
/// or to `*`. Returns the number of clients that received it.
pub async fn broadcast_to_collection(
    hub: &RwLock<WSHub>,
    collection: &str,
    payload: &Value,
) -> usize {
    let text = payload.to_string();
    let msg = Message::Text(text.into());
    let hub = hub.read().await;
    let mut sent = 0;
    for client in hub.clients.values() {
        let subs = client.subs.lock().unwrap_or_else(|e| e.into_inner());
        if (subs.contains_key(SUB_ALL) || subs.contains_key(collection))
            && client.sender.send(msg.clone()).is_ok()
        {
            sent += 1;
        }
    }
    sent
}

/// Rebuilds the filter-eval view from a serialized `Record` (`{id, data,
/// created_at, updated_at}`): merges `data` keys to top level and exposes
/// `created`/`updated` aliases — same shape as `db::record_view`.
fn eval_view(record: &Value) -> Value {
    let mut view = serde_json::Map::new();
    if let Some(obj) = record.as_object() {
        for (k, v) in obj {
            if k != "data" {
                view.insert(k.clone(), v.clone());
            }
        }
        if let Some(data) = obj.get("data").and_then(Value::as_object) {
            for (k, v) in data {
                view.insert(k.clone(), v.clone());
            }
        }
    }
    if let Some(c) = record.get("created_at").and_then(Value::as_str) {
        view.insert("created".into(), Value::String(c.to_string()));
    }
    if let Some(u) = record.get("updated_at").and_then(Value::as_str) {
        view.insert("updated".into(), Value::String(u.to_string()));
    }
    Value::Object(view)
}

/// Broadcasts a record-change event (`create`/`update`/`delete`) honoring
/// per-subscription `filter` expressions (PocketBase realtime filters).
/// `@request.auth.*` resolves to null — WS connections are anonymous.
pub async fn broadcast_record_change(
    hub: &RwLock<WSHub>,
    collection: &str,
    action: &str,
    record: &Value,
    old_record: Option<&Value>,
) -> usize {
    let mut payload = serde_json::json!({
        "type": "record",
        "action": action,
        "collection": collection,
        "record": record,
    });
    if let Some(old) = old_record {
        payload["oldRecord"] = old.clone();
    }
    let view = eval_view(record);
    let text = payload.to_string();
    let msg = Message::Text(text.into());
    let hub = hub.read().await;
    let mut sent = 0;
    for client in hub.clients.values() {
        let subs = client.subs.lock().unwrap_or_else(|e| e.into_inner());
        let matched = if subs.contains_key(SUB_ALL) {
            true
        } else {
            match subs.get(collection) {
                None => false,
                Some(None) => true,
                Some(Some(filter)) => {
                    blazers_core::filter::eval(filter, &view, &Value::Null).unwrap_or(false)
                }
            }
        };
        drop(subs);
        if matched && client.sender.send(msg.clone()).is_ok() {
            sent += 1;
        }
    }
    sent
}

/// Broadcasts a record-change event to all subscribers (no `oldRecord`).
pub async fn broadcast_record(
    hub: &RwLock<WSHub>,
    collection: &str,
    action: &str,
    record: &Value,
) -> usize {
    broadcast_record_change(hub, collection, action, record, None).await
}
