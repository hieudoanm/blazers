use crate::db::*;
use crate::models::*;
use rusqlite::Connection;

#[test]
fn test_user_insert_and_find() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_user(&conn, "u1", "user@example.com", "hashed_pw").unwrap();

    let found = find_user_by_email(&conn, "user@example.com")
        .unwrap()
        .unwrap();
    assert_eq!(found.0, "u1");
    assert_eq!(found.1, "user@example.com");
    assert_eq!(found.4, "hashed_pw");
}

#[test]
fn test_find_user_by_email_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(
        find_user_by_email(&conn, "noone@example.com")
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_insert_user_duplicate_email() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_user(&conn, "u1", "dup@example.com", "pw").unwrap();
    let err = insert_user(&conn, "u2", "dup@example.com", "pw2").unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

// --- Log operations ---
