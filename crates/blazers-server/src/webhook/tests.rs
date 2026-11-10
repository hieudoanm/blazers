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
