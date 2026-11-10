use super::*;

#[test]
fn test_get_schema_fields_empty() {
    assert!(get_schema_fields("").is_empty());
}

#[test]
fn test_get_schema_fields_empty_object() {
    assert!(get_schema_fields("{}").is_empty());
}

#[test]
fn test_get_schema_fields_single_string() {
    let fields = get_schema_fields(r#"{"name": "string"}"#);
    assert_eq!(fields, vec![("name".to_string(), "TEXT".to_string())]);
}

#[test]
fn test_get_schema_fields_optional_strips_question() {
    let fields = get_schema_fields(r#"{"name?": "string"}"#);
    assert_eq!(fields, vec![("name".to_string(), "TEXT".to_string())]);
}

#[test]
fn test_get_schema_fields_type_mappings() {
    let fields: Vec<(String, String)> =
        get_schema_fields(r#"{"a": "string", "b": "number", "c": "integer", "d": "boolean"}"#);
    assert_eq!(fields.len(), 4);
    let map: std::collections::HashMap<&str, &str> = fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    assert_eq!(map.get("a"), Some(&"TEXT"));
    assert_eq!(map.get("b"), Some(&"REAL"));
    assert_eq!(map.get("c"), Some(&"INTEGER"));
    assert_eq!(map.get("d"), Some(&"INTEGER"));
}

#[test]
fn test_get_schema_fields_unknown_type_defaults_to_text() {
    let fields = get_schema_fields(r#"{"x": "unknown_type"}"#);
    assert_eq!(fields, vec![("x".to_string(), "TEXT".to_string())]);
}

#[test]
fn test_get_schema_fields_mixed_types() {
    let fields =
        get_schema_fields(r#"{"email": "email", "url": "url", "tags": "array", "meta": "object"}"#);
    let map: std::collections::HashMap<&str, &str> = fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    assert_eq!(map.len(), 4);
    for v in map.values() {
        assert_eq!(*v, "TEXT");
    }
}

#[test]
fn test_build_search_clause_empty() {
    assert_eq!(build_search_clause(""), None);
}

#[test]
fn test_build_search_clause_whitespace() {
    assert_eq!(build_search_clause("   "), None);
}

#[test]
fn test_build_search_clause_single_word() {
    let (clause, params) = build_search_clause("hello").unwrap();
    assert_eq!(clause, "data LIKE ?1");
    assert_eq!(params, vec!["%hello%"]);
}

#[test]
fn test_build_search_clause_multiple_words() {
    let (clause, params) = build_search_clause("hello world").unwrap();
    assert_eq!(clause, "data LIKE ?1 AND data LIKE ?2");
    assert_eq!(params, vec!["%hello%", "%world%"]);
}

#[test]
fn test_build_search_clause_special_characters() {
    let (clause, params) = build_search_clause("foo%_bar").unwrap();
    assert_eq!(clause, "data LIKE ?1");
    assert_eq!(params, vec!["%foo%_bar%"]);
}

#[test]
fn test_data_dir_default() {
    unsafe {
        std::env::set_var("BLAZERS_DATA", "/tmp/test_blazers");
    }
    let d = data_dir();
    assert_eq!(d, std::path::PathBuf::from("/tmp/test_blazers"));
    unsafe {
        std::env::remove_var("BLAZERS_DATA");
    }
}

#[test]
fn test_data_dir_fallback_home() {
    unsafe {
        std::env::remove_var("BLAZERS_DATA");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let d = data_dir();
    assert_eq!(d, std::path::PathBuf::from(home).join(".blazers"));
}

#[test]
fn test_insert_get_list_delete_collection() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "test_coll", r#"{"name": "string"}"#).unwrap();

    let col = get_collection(&conn, "test_coll").unwrap().unwrap();
    assert_eq!(col.name, "test_coll");
    assert_eq!(col.schema, r#"{"name": "string"}"#);

    let cols = list_collections(&conn).unwrap();
    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "test_coll");

    delete_collection(&conn, "test_coll").unwrap();
    assert!(get_collection(&conn, "test_coll").unwrap().is_none());
    assert!(list_collections(&conn).unwrap().is_empty());
}

#[test]
fn test_insert_collection_duplicate() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "dup", "{}").unwrap();
    let err = insert_collection(&conn, "dup", "{}").unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

#[test]
fn test_get_collection_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(get_collection(&conn, "nonexistent").unwrap().is_none());
}

#[test]
fn test_list_collections_empty() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    let cols = list_collections(&conn).unwrap();
    assert!(cols.is_empty());
}

#[test]
fn test_create_collection_table_and_migrate() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "schema_test", r#"{"name": "string"}"#).unwrap();
    create_collection_table(&conn, "schema_test", r#"{"name": "string"}"#).unwrap();

    migrate_collection_schema(
        &conn,
        "schema_test",
        r#"{"name": "string"}"#,
        r#"{"name": "string", "age": "integer"}"#,
    )
    .unwrap();

    let col = get_collection(&conn, "schema_test").unwrap().unwrap();
    assert_eq!(col.schema, r#"{"name": "string", "age": "integer"}"#);
}

#[test]
fn test_update_collection_with_new_schema() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "upd_coll", r#"{"name": "string"}"#).unwrap();
    create_collection_table(&conn, "upd_coll", r#"{"name": "string"}"#).unwrap();

    let updated = update_collection(
        &conn,
        "upd_coll",
        Some(r#"{"name": "string", "active": "boolean"}"#),
    )
    .unwrap();
    assert!(updated.schema.contains("active"));
}
