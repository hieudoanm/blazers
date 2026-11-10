use crate::db::*;
use rusqlite::Connection;

#[test]
fn test_notification_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let n = insert_notification(&conn, "n1", "Hello", "Body text", "info").unwrap();
    assert_eq!(n.title, "Hello");
    assert!(!n.is_read);

    let fetched = get_notification(&conn, "n1").unwrap().unwrap();
    assert_eq!(fetched.body, "Body text");

    let list = list_notifications(&conn).unwrap();
    assert_eq!(list.len(), 1);

    let read = update_notification_read(&conn, "n1").unwrap();
    assert!(read.is_read);

    let deleted = delete_notification(&conn, "n1").unwrap();
    assert!(deleted);
    assert!(get_notification(&conn, "n1").unwrap().is_none());

    let deleted_false = delete_notification(&conn, "nonexistent").unwrap();
    assert!(!deleted_false);
}

#[test]
fn test_notification_clear_all() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_notification(&conn, "n1", "A", "", "info").unwrap();
    insert_notification(&conn, "n2", "B", "", "warn").unwrap();

    clear_notifications(&conn).unwrap();
    let list = list_notifications(&conn).unwrap();
    assert!(list.is_empty());
}

// --- Webhook CRUD ---

#[test]
fn test_get_notification_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(get_notification(&conn, "nonexistent").unwrap().is_none());
}
