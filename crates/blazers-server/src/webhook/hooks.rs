//! Synchronous record hooks — the JSVM-lite escape hatch. Webhooks
//! subscribed to `record.before_create|update|delete` are called *before*
//! the DB write; a 2xx JSON response may return `{"record": {...}}` to
//! merge fields into the record data, or `{"cancel": true}` to abort.
//! Non-2xx responses abort the request.

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use crate::handlers::AppState;
use blazers_core::db;
use blazers_core::models::*;

use super::{EVENT_RECORD_BEFORE_CREATE, EVENT_RECORD_BEFORE_DELETE, EVENT_RECORD_BEFORE_UPDATE};

pub enum HookOutcome {
    /// No hook objected; `Some(fields)` merges into the record `data`.
    Continue(Option<Value>),
    /// A hook returned `{"cancel": true}` or a non-2xx status.
    Cancel,
}

/// Runs all active `record.before_*` webhooks for `event` and aggregates
/// their `record` patches. `record` is the record *view* JSON.
pub async fn run_record_hooks(
    state: &Arc<AppState>,
    event: &str,
    collection: &str,
    record: &Value,
    auth: &Value,
) -> std::result::Result<HookOutcome, AppError> {
    let hooks = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::list_webhooks(&conn).unwrap_or_default()
    };
    let hooks: Vec<Webhook> = hooks
        .into_iter()
        .filter(|h| h.is_active && h.events.iter().any(|e| e == event))
        .collect();
    if hooks.is_empty() {
        return Ok(HookOutcome::Continue(None));
    }

    let payload = serde_json::json!({
        "event": event,
        "data": { "collection": collection, "record": record, "auth": auth },
    });
    let body = serde_json::to_string(&payload)
        .map_err(|e| AppError::Internal(format!("hook payload: {e}")))?;

    let mut merged: Option<Value> = None;
    for hook in hooks {
        let resp = state
            .http_client
            .post(&hook.url)
            .timeout(Duration::from_secs(5))
            .header("Content-Type", "application/json")
            .header("X-Webhook-ID", &hook.id)
            .header("X-Webhook-Event", event)
            .body(body.clone())
            .send()
            .await;
        let resp = match resp {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                log_hook(state, &hook, event, &body, r.status().as_u16() as i64, "").await;
                return Ok(HookOutcome::Cancel);
            }
            Err(_) => {
                log_hook(state, &hook, event, &body, 0, "connection failed").await;
                return Ok(HookOutcome::Cancel);
            }
        };
        let text = resp.text().await.unwrap_or_default();
        log_hook(state, &hook, event, &body, 200, &text).await;
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        if v.get("cancel").and_then(Value::as_bool) == Some(true) {
            return Ok(HookOutcome::Cancel);
        }
        if let Some(fields) = v.get("record").filter(|r| r.is_object()) {
            merge_patch(&mut merged, fields.clone());
        }
    }
    Ok(HookOutcome::Continue(merged))
}

/// Shallow-merges a hook-returned `record` patch into a record `data` object.
pub fn merge_data_patch(data: &mut Value, patch: Value) {
    if let (Value::Object(d), Value::Object(p)) = (data, patch) {
        for (k, v) in p {
            d.insert(k, v);
        }
    }
}

fn merge_patch(acc: &mut Option<Value>, patch: Value) {
    match acc {
        Some(Value::Object(a)) => {
            if let Value::Object(p) = patch {
                for (k, v) in p {
                    a.insert(k, v);
                }
            }
        }
        slot => *slot = Some(patch),
    }
}

async fn log_hook(
    state: &Arc<AppState>,
    hook: &Webhook,
    event: &str,
    body: &str,
    status: i64,
    resp_body: &str,
) {
    let log = WebhookLog {
        id: uuid::Uuid::new_v4().to_string().replace('-', ""),
        webhook_id: hook.id.clone(),
        event: event.to_string(),
        url: hook.url.clone(),
        request_body: body.to_string(),
        response_status: status,
        response_body: resp_body.to_string(),
        error: String::new(),
        status: if (200..300).contains(&status) {
            "success".into()
        } else {
            "failure".into()
        },
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    if let Ok(conn) = state.db.get().await {
        let _ = db::insert_webhook_log(&conn, &log);
    }
}

/// Convenience: event name for a record action.
pub fn before_event(action: &str) -> &'static str {
    match action {
        "create" => EVENT_RECORD_BEFORE_CREATE,
        "update" => EVENT_RECORD_BEFORE_UPDATE,
        _ => EVENT_RECORD_BEFORE_DELETE,
    }
}
