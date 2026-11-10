use super::*;

#[test]
fn register_request_defaults_empty_strings() {
    let req: RegisterRequest = serde_json::from_value(json!({})).unwrap();
    assert_eq!(req.email, "");
    assert_eq!(req.password, "");
}

#[test]
fn empty_records_page() {
    let page = RecordsPage {
        records: vec![],
        total: 0,
        page: 1,
        per_page: 20,
        total_pages: 0,
    };
    let v = serde_json::to_value(&page).unwrap();
    let d: RecordsPage = serde_json::from_value(v).unwrap();
    assert!(d.records.is_empty());
    assert_eq!(d.total, 0);
    assert_eq!(d.total_pages, 0);
}

#[test]
fn cache_entry_zero_ttl() {
    let entry = CacheEntry {
        key: "k".into(),
        value: "v".into(),
        ttl: 0,
        expires_at: "2024-01-01T00:00:00Z".into(),
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&entry).unwrap();
    let d: CacheEntry = serde_json::from_value(v).unwrap();
    assert_eq!(d.ttl, 0);
}

#[test]
fn notification_empty_body() {
    let n = Notification {
        id: "n3".into(),
        title: "Empty".into(),
        body: "".into(),
        ntype: "warn".into(),
        is_read: false,
        created_at: "2024-01-01T00:00:00Z".into(),
    };
    let v = serde_json::to_value(&n).unwrap();
    let d: Notification = serde_json::from_value(v).unwrap();
    assert_eq!(d.body, "");
}
