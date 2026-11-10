use crate::db::*;
use crate::models::*;
use rusqlite::Connection;
use serde_json::json;

#[test]
fn test_record_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "records", "{}").unwrap();
    create_collection_table(&conn, "records", "{}").unwrap();

    let data = serde_json::json!({"title": "hello", "count": 42});
    let rec = insert_record(&conn, "records", "rec1", &data).unwrap();
    assert_eq!(rec.id, "rec1");
    assert_eq!(rec.data["title"], "hello");

    let fetched = get_record(&conn, "records", "rec1").unwrap().unwrap();
    assert_eq!(fetched.id, "rec1");
    assert_eq!(fetched.data["count"], 42);

    let updated_data = serde_json::json!({"title": "updated", "count": 99});
    let updated = update_record(&conn, "records", "rec1", &updated_data).unwrap();
    assert_eq!(updated.data["title"], "updated");

    let page = list_records(&conn, "records", 1, 10, "").unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.len(), 1);

    delete_record(&conn, "records", "rec1").unwrap();
    assert!(get_record(&conn, "records", "rec1").unwrap().is_none());
}

#[test]
fn test_record_search() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "search_test", "{}").unwrap();
    create_collection_table(&conn, "search_test", "{}").unwrap();

    insert_record(
        &conn,
        "search_test",
        "a",
        &serde_json::json!({"text": "hello world"}),
    )
    .unwrap();
    insert_record(
        &conn,
        "search_test",
        "b",
        &serde_json::json!({"text": "goodbye world"}),
    )
    .unwrap();

    let page = list_records(&conn, "search_test", 1, 10, "hello").unwrap();
    assert_eq!(page.total, 1);

    let page2 = list_records(&conn, "search_test", 1, 10, "world").unwrap();
    assert_eq!(page2.total, 2);
}

#[test]
fn test_record_pagination() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "pagination", "{}").unwrap();
    create_collection_table(&conn, "pagination", "{}").unwrap();

    for i in 0..5 {
        insert_record(
            &conn,
            "pagination",
            &format!("r{i}"),
            &serde_json::json!({"n": i}),
        )
        .unwrap();
    }

    let page = list_records(&conn, "pagination", 1, 2, "").unwrap();
    assert_eq!(page.records.len(), 2);
    assert_eq!(page.total, 5);
    assert_eq!(page.total_pages, 3);
}

#[test]
fn test_get_record_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_collection(&conn, "nf_test", "{}").unwrap();
    create_collection_table(&conn, "nf_test", "{}").unwrap();

    assert!(
        get_record(&conn, "nf_test", "nonexistent")
            .unwrap()
            .is_none()
    );
}

// --- Bucket CRUD ---

#[test]
fn test_update_record_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    insert_collection(&conn, "rc", "{}").unwrap();
    create_collection_table(&conn, "rc", "{}").unwrap();
    let err = update_record(&conn, "rc", "noexist", &json!({"x": 1})).unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[test]
fn test_insert_record_duplicate() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    insert_collection(&conn, "dup_rec", "{}").unwrap();
    create_collection_table(&conn, "dup_rec", "{}").unwrap();
    insert_record(&conn, "dup_rec", "r1", &json!({"x": 1})).unwrap();
    let err = insert_record(&conn, "dup_rec", "r1", &json!({"x": 2})).unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}
