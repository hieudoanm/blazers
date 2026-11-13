use super::*;

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
