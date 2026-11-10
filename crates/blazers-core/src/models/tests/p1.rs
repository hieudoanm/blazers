use super::*;

#[test]
fn default_schema_value() {
    assert_eq!(super::collections::default_schema(), "{}");
}

#[test]
fn default_data_value() {
    assert_eq!(super::collections::default_data(), json!({}));
}

#[test]
fn app_error_bad_request() {
    let resp = AppError::BadRequest("bad".into()).into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn app_error_not_found() {
    let resp = AppError::NotFound("nf".into()).into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[test]
fn app_error_conflict() {
    let resp = AppError::Conflict("conflict".into()).into_response();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[test]
fn app_error_unauthorized() {
    let resp = AppError::Unauthorized("unauth".into()).into_response();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn app_error_forbidden() {
    let resp = AppError::Forbidden("forbid".into()).into_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[test]
fn app_error_internal() {
    let resp = AppError::Internal("err".into()).into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn record_round_trip() {
    let record = Record {
        id: "rec_1".into(),
        data: json!({"name": "test"}),
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&record).unwrap();
    let d: Record = serde_json::from_value(v).unwrap();
    assert_eq!(record.id, d.id);
    assert_eq!(record.data, d.data);
}

#[test]
fn records_page_round_trip() {
    let record = Record {
        id: "rec_1".into(),
        data: json!({"x": 1}),
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let page = RecordsPage {
        records: vec![record],
        total: 1,
        page: 1,
        per_page: 20,
        total_pages: 1,
    };
    let v = serde_json::to_value(&page).unwrap();
    let d: RecordsPage = serde_json::from_value(v).unwrap();
    assert_eq!(page.total, d.total);
    assert_eq!(page.records.len(), d.records.len());
}

#[test]
fn login_response_serialization() {
    let user = UserResponse {
        id: "u1".into(),
        email: "a@b.com".into(),
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let resp = LoginResponse {
        token: "tok_1".into(),
        user,
    };
    let v = serde_json::to_value(&resp).unwrap();
    assert_eq!(v["token"], "tok_1");
    assert_eq!(v["user"]["email"], "a@b.com");
}

#[test]
fn bucket_round_trip() {
    let bucket = Bucket {
        name: "b".into(),
        is_public: true,
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&bucket).unwrap();
    let d: Bucket = serde_json::from_value(v).unwrap();
    assert_eq!(bucket.name, d.name);
    assert_eq!(bucket.is_public, d.is_public);
}

#[test]
fn file_record_round_trip() {
    let file = FileRecord {
        id: "f1".into(),
        bucket: "b".into(),
        filename: "t.txt".into(),
        mime_type: "text/plain".into(),
        size: 100,
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&file).unwrap();
    let d: FileRecord = serde_json::from_value(v).unwrap();
    assert_eq!(file.id, d.id);
    assert_eq!(file.size, d.size);
}

#[test]
fn cache_entry_round_trip() {
    let entry = CacheEntry {
        key: "k".into(),
        value: "v".into(),
        ttl: 3600,
        expires_at: "2024-01-02T00:00:00Z".into(),
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&entry).unwrap();
    let d: CacheEntry = serde_json::from_value(v).unwrap();
    assert_eq!(entry.key, d.key);
    assert_eq!(entry.ttl, d.ttl);
}

#[test]
fn notification_round_trip() {
    let n = Notification {
        id: "n1".into(),
        title: "Hi".into(),
        body: "Body".into(),
        ntype: "info".into(),
        is_read: false,
        created_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&n).unwrap();
    let d: Notification = serde_json::from_value(v).unwrap();
    assert_eq!(n.id, d.id);
    assert_eq!(n.ntype, d.ntype);
}

#[test]
fn notification_serializes_ntype_as_type() {
    let n = Notification {
        id: "n2".into(),
        title: "T".into(),
        body: "B".into(),
        ntype: "alert".into(),
        is_read: true,
        created_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&n).unwrap();
    assert_eq!(v["type"], "alert");
    assert!(v.get("ntype").is_none());
}

#[test]
fn create_notification_request_deserializes_type() {
    let req: CreateNotificationRequest =
        serde_json::from_value(json!({"title": "Test", "type": "info"})).unwrap();
    assert_eq!(req.title, "Test");
    assert_eq!(req.ntype, "info");
}

#[test]
fn create_collection_request_defaults_schema() {
    let req: CreateCollectionRequest = serde_json::from_value(json!({"name": "c"})).unwrap();
    assert_eq!(req.name, "c");
    assert_eq!(req.schema, "{}");
}

#[test]
fn create_record_request_defaults_data() {
    let req: CreateRecordRequest = serde_json::from_value(json!({"id": "r1"})).unwrap();
    assert_eq!(req.id.unwrap(), "r1");
    assert_eq!(req.data, json!({}));
}
