use crate::db::*;
use rusqlite::Connection;

#[test]
fn test_cache_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let entry = set_cache_entry(&conn, "k1", "v1", 0).unwrap();
    assert_eq!(entry.key, "k1");
    assert_eq!(entry.ttl, 0);

    let fetched = get_cache_entry(&conn, "k1").unwrap().unwrap();
    assert_eq!(fetched.value, "v1");

    let deleted = delete_cache_entry(&conn, "k1").unwrap();
    assert!(deleted);
    assert!(get_cache_entry(&conn, "k1").unwrap().is_none());

    let deleted2 = delete_cache_entry(&conn, "nonexistent").unwrap();
    assert!(!deleted2);
}

#[test]
fn test_cache_list_and_flush() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    set_cache_entry(&conn, "a", "1", 0).unwrap();
    set_cache_entry(&conn, "b", "2", 0).unwrap();

    let entries = list_cache_entries(&conn).unwrap();
    assert_eq!(entries.len(), 2);

    flush_cache(&conn).unwrap();
    assert!(list_cache_entries(&conn).unwrap().is_empty());
}

#[test]
fn test_cache_upsert() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    set_cache_entry(&conn, "k", "old", 0).unwrap();
    set_cache_entry(&conn, "k", "new", 3600).unwrap();

    let entry = get_cache_entry(&conn, "k").unwrap().unwrap();
    assert_eq!(entry.value, "new");
    assert_eq!(entry.ttl, 3600);
}

// --- Notification CRUD ---

#[test]
fn test_get_cache_entry_expired() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    set_cache_entry(&conn, "expkey", "oldval", 1).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));
    let result = get_cache_entry(&conn, "expkey").unwrap();
    assert!(result.is_none());
}
