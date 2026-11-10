use crate::db::*;
use crate::models::*;
use rusqlite::Connection;

#[test]
fn test_bucket_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let bucket = insert_bucket(&conn, "my_bucket", true).unwrap();
    assert_eq!(bucket.name, "my_bucket");
    assert!(bucket.is_public);

    let fetched = get_bucket(&conn, "my_bucket").unwrap().unwrap();
    assert_eq!(fetched.name, "my_bucket");
    assert!(fetched.is_public);

    let buckets = list_buckets(&conn).unwrap();
    assert_eq!(buckets.len(), 1);

    let files = delete_bucket(&conn, "my_bucket").unwrap();
    assert!(files.is_empty());
    assert!(get_bucket(&conn, "my_bucket").unwrap().is_none());
}

#[test]
fn test_bucket_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(get_bucket(&conn, "nonexistent").unwrap().is_none());
}

#[test]
fn test_bucket_duplicate() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_bucket(&conn, "dup_bucket", false).unwrap();
    let err = insert_bucket(&conn, "dup_bucket", false).unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

// --- File CRUD ---

#[test]
fn test_file_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_bucket(&conn, "files_bucket", false).unwrap();

    let file = insert_file(&conn, "files_bucket", "f1", "test.txt", "text/plain", 100).unwrap();
    assert_eq!(file.id, "f1");
    assert_eq!(file.filename, "test.txt");

    let fetched = get_file(&conn, "f1").unwrap().unwrap();
    assert_eq!(fetched.bucket, "files_bucket");

    let page = list_files(&conn, "files_bucket", 1, 10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.files.len(), 1);

    let deleted = delete_file(&conn, "f1").unwrap().unwrap();
    assert_eq!(deleted.id, "f1");
    assert!(get_file(&conn, "f1").unwrap().is_none());
}

#[test]
fn test_delete_file_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(delete_file(&conn, "nonexistent").unwrap().is_none());
}

// --- Cache CRUD ---

#[test]
fn test_delete_bucket_with_files() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    insert_bucket(&conn, "bf", false).unwrap();
    insert_file(&conn, "bf", "f1", "a.txt", "text/plain", 10).unwrap();
    let files = delete_bucket(&conn, "bf").unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].id, "f1");
}

#[test]
fn test_list_files_empty() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    let page = list_files(&conn, "nonexistent", 1, 10).unwrap();
    assert!(page.files.is_empty());
    assert_eq!(page.total, 0);
}
