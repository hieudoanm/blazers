use crate::db::*;
use rusqlite::Connection;

#[test]
fn test_ws_connection_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_ws_connection(&conn, "ws1", "127.0.0.1", "/chat", "test-agent").unwrap();

    let conns = list_ws_connections(&conn).unwrap();
    assert_eq!(conns.len(), 1);
    assert!(conns[0].is_active);

    let fetched = get_ws_connection(&conn, "ws1").unwrap().unwrap();
    assert_eq!(fetched.remote_addr, "127.0.0.1");

    update_ws_disconnect(&conn, "ws1").unwrap();
    let after = get_ws_connection(&conn, "ws1").unwrap().unwrap();
    assert!(!after.is_active);
    assert!(!after.disconnected_at.is_empty());

    delete_ws_connection(&conn, "ws1").unwrap();
    assert!(get_ws_connection(&conn, "ws1").unwrap().is_none());
}

#[test]
fn test_get_ws_connection_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(get_ws_connection(&conn, "nonexistent").unwrap().is_none());
}

// --- WS Messages ---

#[test]
fn test_ws_message_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_ws_connection(&conn, "ws_msg", "10.0.0.1", "/", "agent").unwrap();
    insert_ws_message(&conn, "ws_msg", "sent", "hello").unwrap();
    insert_ws_message(&conn, "ws_msg", "received", "world").unwrap();

    let msgs = list_ws_messages(&conn, "ws_msg").unwrap();
    assert_eq!(msgs.len(), 2);

    let all = list_all_ws_messages(&conn).unwrap();
    assert_eq!(all.len(), 2);
}

// --- PubSub Topic CRUD ---

#[test]
fn test_list_ws_messages_empty() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(list_ws_messages(&conn, "nonexistent").unwrap().is_empty());
}
