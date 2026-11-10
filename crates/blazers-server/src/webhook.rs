use std::sync::Arc;

use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::Value;
use sha2::Sha256;
use uuid::Uuid;

use crate::handlers::AppState;
use blazers_core::db;
use blazers_core::models::*;

type HmacSha256 = Hmac<Sha256>;

pub const EVENT_RECORD_CREATE: &str = "record.create";
pub const EVENT_RECORD_UPDATE: &str = "record.update";
pub const EVENT_RECORD_DELETE: &str = "record.delete";
pub const EVENT_COLLECTION_CREATE: &str = "collection.create";
pub const EVENT_COLLECTION_DELETE: &str = "collection.delete";
pub const EVENT_BUCKET_CREATE: &str = "bucket.create";
pub const EVENT_BUCKET_DELETE: &str = "bucket.delete";
pub const EVENT_SECRET_CREATE: &str = "secret.create";
pub const EVENT_SECRET_UPDATE: &str = "secret.update";
pub const EVENT_SECRET_DELETE: &str = "secret.delete";
pub const EVENT_NOTIFICATION_CREATE: &str = "notification.create";
pub const EVENT_LOG_CREATE: &str = "log.create";
pub const EVENT_PUBSUB_TOPIC_CREATE: &str = "pubsub.topic.create";
pub const EVENT_PUBSUB_TOPIC_DELETE: &str = "pubsub.topic.delete";
pub const EVENT_PUBSUB_MESSAGE_CREATE: &str = "pubsub.message.create";

pub fn validate_events(events: &[String]) -> std::result::Result<(), String> {
    for e in events {
        match e.as_str() {
            EVENT_RECORD_CREATE
            | EVENT_RECORD_UPDATE
            | EVENT_RECORD_DELETE
            | EVENT_COLLECTION_CREATE
            | EVENT_COLLECTION_DELETE
            | EVENT_BUCKET_CREATE
            | EVENT_BUCKET_DELETE
            | EVENT_SECRET_CREATE
            | EVENT_SECRET_UPDATE
            | EVENT_SECRET_DELETE
            | EVENT_NOTIFICATION_CREATE
            | EVENT_LOG_CREATE
            | EVENT_PUBSUB_TOPIC_CREATE
            | EVENT_PUBSUB_TOPIC_DELETE
            | EVENT_PUBSUB_MESSAGE_CREATE
            | crate::cronjobs::EVENT_CRONJOB_CREATE
            | crate::cronjobs::EVENT_CRONJOB_UPDATE
            | crate::cronjobs::EVENT_CRONJOB_DELETE => {}
            _ => return Err(format!("unknown event: {e}")),
        }
    }
    Ok(())
}

pub fn webhook_record_data(collection: &str, record: &Record) -> Value {
    serde_json::json!({
        "collection": collection,
        "record": record,
    })
}

pub fn webhook_collection_data(collection: &Collection) -> Value {
    serde_json::json!({
        "collection": collection,
    })
}

pub fn webhook_bucket_data(bucket: &Bucket) -> Value {
    serde_json::json!({
        "bucket": bucket,
    })
}

pub fn dispatch_event(state: &Arc<AppState>, event: &str, data: Value) {
    let state = Arc::clone(state);
    let event = event.to_string();
    tokio::spawn(async move {
        let hooks = {
            let conn = state.db.get().await.unwrap();
            db::list_webhooks(&conn).unwrap_or_default()
        };

        let now = Utc::now().to_rfc3339();
        let payload = serde_json::json!({
            "event": event,
            "created_at": now,
            "data": data,
        });
        let body = serde_json::to_string(&payload).unwrap_or_default();

        for h in hooks {
            if !h.is_active || !h.events.contains(&event) {
                continue;
            }
            let hook = h;
            let body = body.clone();
            let event = event.clone();
            let state = Arc::clone(&state);

            tokio::spawn(async move {
                send_webhook(&state, &hook, &event, &body).await;
            });
        }
    });
}

async fn send_webhook(state: &Arc<AppState>, hook: &Webhook, event: &str, body_str: &str) {
    let client = &state.http_client;

    let mut req_builder = client
        .post(&hook.url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-ID", &hook.id)
        .header("X-Webhook-Event", event);

    if !hook.secret.is_empty() {
        let mut mac = HmacSha256::new_from_slice(hook.secret.as_bytes()).expect("HMAC key");
        mac.update(body_str.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        req_builder = req_builder.header("X-Webhook-Signature-256", sig);
    }

    let (status_code, resp_body, error_msg, status) = match req_builder.send().await {
        Ok(resp) => {
            let sc = resp.status().as_u16() as i64;
            let rb = resp.text().await.unwrap_or_default();
            let st = if (200..300).contains(&sc) {
                "success"
            } else {
                "failure"
            };
            (sc, rb, String::new(), st.to_string())
        }
        Err(e) => (0, String::new(), e.to_string(), "failure".to_string()),
    };

    let log = WebhookLog {
        id: Uuid::new_v4().to_string().replace('-', ""),
        webhook_id: hook.id.clone(),
        event: event.to_string(),
        url: hook.url.clone(),
        request_body: body_str.to_string(),
        response_status: status_code,
        response_body: resp_body,
        error: error_msg,
        status,
        created_at: Utc::now().to_rfc3339(),
    };

    if let Ok(conn) = state.db.get().await {
        let _ = db::insert_webhook_log(&conn, &log);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_events_single_valid() {
        assert!(validate_events(&["record.create".to_string()]).is_ok());
    }

    #[test]
    fn test_validate_events_multiple_valid() {
        let events = vec![
            "record.create".to_string(),
            "record.update".to_string(),
            "record.delete".to_string(),
        ];
        assert!(validate_events(&events).is_ok());
    }

    #[test]
    fn test_validate_events_wildcard() {
        let result = validate_events(&["*".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown event"));
    }

    #[test]
    fn test_validate_events_all_known() {
        let events = vec![
            "record.create".to_string(),
            "record.update".to_string(),
            "record.delete".to_string(),
            "collection.create".to_string(),
            "collection.delete".to_string(),
            "bucket.create".to_string(),
            "bucket.delete".to_string(),
            "secret.create".to_string(),
            "secret.update".to_string(),
            "secret.delete".to_string(),
            "notification.create".to_string(),
            "log.create".to_string(),
            "pubsub.topic.create".to_string(),
            "pubsub.topic.delete".to_string(),
            "pubsub.message.create".to_string(),
            "cronjob.create".to_string(),
            "cronjob.update".to_string(),
            "cronjob.delete".to_string(),
        ];
        assert!(validate_events(&events).is_ok());
    }

    #[test]
    fn test_validate_events_invalid() {
        let result = validate_events(&["bogus.event".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown event"));
    }

    #[test]
    fn test_validate_events_empty() {
        assert!(validate_events(&[]).is_ok());
    }

    #[test]
    fn test_validate_events_mixed_valid_and_invalid() {
        let result = validate_events(&["record.create".to_string(), "nope".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown event"));
    }

    #[test]
    fn test_webhook_record_data() {
        let record = Record {
            id: "rec_123".to_string(),
            data: json!({"title": "hello", "count": 42}),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };
        let result = webhook_record_data("posts", &record);
        assert_eq!(result["collection"], "posts");
        assert_eq!(result["record"]["id"], "rec_123");
        assert_eq!(result["record"]["data"]["title"], "hello");
        assert_eq!(result["record"]["data"]["count"], 42);
        assert_eq!(result["record"]["created_at"], "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_webhook_collection_data() {
        let collection = Collection {
            name: "posts".to_string(),
            schema: "{\"title\":\"string\"}".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };
        let result = webhook_collection_data(&collection);
        assert_eq!(result["collection"]["name"], "posts");
        assert_eq!(result["collection"]["schema"], "{\"title\":\"string\"}");
    }

    #[test]
    fn test_webhook_bucket_data() {
        let bucket = Bucket {
            name: "avatars".to_string(),
            is_public: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };
        let result = webhook_bucket_data(&bucket);
        assert_eq!(result["bucket"]["name"], "avatars");
        assert_eq!(result["bucket"]["is_public"], true);
    }
}
