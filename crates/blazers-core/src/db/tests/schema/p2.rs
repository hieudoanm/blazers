use super::*;

#[test]
fn test_update_collection_no_schema_change() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "nochange", r#"{"x": "string"}"#).unwrap();
    create_collection_table(&conn, "nochange", r#"{"x": "string"}"#).unwrap();

    let updated = update_collection(&conn, "nochange", None).unwrap();
    assert_eq!(updated.schema, r#"{"x": "string"}"#);
}

#[test]
fn test_sync_schema_columns() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "sync_coll", r#"{"val": "string"}"#).unwrap();
    create_collection_table(&conn, "sync_coll", r#"{"val": "string"}"#).unwrap();

    let rec = insert_record(
        &conn,
        "sync_coll",
        "r1",
        &serde_json::json!({"val": "hello"}),
    )
    .unwrap();
    assert_eq!(rec.id, "r1");

    sync_schema_columns(&conn, "sync_coll", "r1", r#"{"val": "string"}"#).unwrap();
    let fetched = get_record(&conn, "sync_coll", "r1").unwrap().unwrap();
    assert_eq!(fetched.data["val"], "hello");
}

#[test]
fn test_update_collection_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    let err = update_collection(&conn, "nonexistent", Some("{}")).unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}
