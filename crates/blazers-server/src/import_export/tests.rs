use super::*;
use serde_json::json;

#[test]
fn export_data_empty_round_trip() {
    let data = ExportData {
        collections: vec![],
        records: HashMap::new(),
        buckets: vec![],
        files: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    let deserialized: ExportData = serde_json::from_str(&json).unwrap();
    assert!(deserialized.collections.is_empty());
    assert!(deserialized.records.is_empty());
    assert!(deserialized.buckets.is_empty());
    assert!(deserialized.files.is_empty());
}

#[test]
fn export_data_with_data_round_trip() {
    let data = ExportData {
        collections: vec![Collection {
            name: "users".into(),
            schema: "{\"name\":\"string\"}".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }],
        records: {
            let mut map = HashMap::new();
            map.insert(
                "users".into(),
                vec![Record {
                    id: "rec_1".into(),
                    data: json!({"name": "Alice"}),
                    created_at: "2024-01-01T00:00:00Z".into(),
                    updated_at: "2024-01-01T00:00:00Z".into(),
                }],
            );
            map
        },
        buckets: vec![Bucket {
            name: "files".into(),
            is_public: true,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }],
        files: vec![FileRecord {
            id: "f1".into(),
            bucket: "files".into(),
            filename: "test.txt".into(),
            mime_type: "text/plain".into(),
            size: 100,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }],
    };
    let json = serde_json::to_string(&data).unwrap();
    let deserialized: ExportData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.collections.len(), 1);
    assert_eq!(deserialized.collections[0].name, "users");
    assert_eq!(deserialized.records.len(), 1);
    assert_eq!(deserialized.records["users"].len(), 1);
    assert_eq!(deserialized.records["users"][0].data["name"], "Alice");
    assert_eq!(deserialized.buckets.len(), 1);
    assert_eq!(deserialized.buckets[0].name, "files");
    assert!(deserialized.buckets[0].is_public);
    assert_eq!(deserialized.files.len(), 1);
    assert_eq!(deserialized.files[0].filename, "test.txt");
    assert_eq!(deserialized.files[0].size, 100);
}

#[test]
fn export_data_json_structure() {
    let data = ExportData {
        collections: vec![],
        records: HashMap::new(),
        buckets: vec![],
        files: vec![],
    };
    let value = serde_json::to_value(&data).unwrap();
    assert!(value.is_object());
    assert!(value.get("collections").unwrap().is_array());
    assert!(value.get("records").unwrap().is_object());
    assert!(value.get("buckets").unwrap().is_array());
    assert!(value.get("files").unwrap().is_array());
    assert_eq!(value["collections"].as_array().unwrap().len(), 0);
    assert_eq!(value["records"].as_object().unwrap().len(), 0);
    assert_eq!(value["buckets"].as_array().unwrap().len(), 0);
    assert_eq!(value["files"].as_array().unwrap().len(), 0);
}

#[test]
fn import_query_default_skip_existing() {
    let query: ImportQuery = serde_json::from_str("{}").unwrap();
    assert_eq!(query.skip_existing, None);
}

#[test]
fn import_query_skip_existing_true() {
    let query: ImportQuery = serde_json::from_str(r#"{"skip_existing": true}"#).unwrap();
    assert_eq!(query.skip_existing, Some(true));
}

#[test]
fn import_query_skip_existing_false() {
    let query: ImportQuery = serde_json::from_str(r#"{"skip_existing": false}"#).unwrap();
    assert_eq!(query.skip_existing, Some(false));
}
