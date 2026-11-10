use crate::db::*;
use rusqlite::Connection;

#[test]
fn test_secret_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let secret = insert_secret(&conn, "s1", "api_key", "abc123", "general").unwrap();
    assert_eq!(secret.name, "api_key");

    let secrets = list_secrets(&conn).unwrap();
    assert_eq!(secrets.len(), 1);

    let fetched = get_secret(&conn, "s1").unwrap().unwrap();
    assert_eq!(fetched.value, "abc123");

    let updated = update_secret(&conn, "s1", "new_key", "xyz789", "admin").unwrap();
    assert_eq!(updated.name, "new_key");

    let confirm = get_secret(&conn, "s1").unwrap().unwrap();
    assert_eq!(confirm.value, "xyz789");

    delete_secret(&conn, "s1").unwrap();
    assert!(get_secret(&conn, "s1").unwrap().is_none());
}

#[test]
fn test_get_secret_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(get_secret(&conn, "nonexistent").unwrap().is_none());
}

// --- User operations ---
