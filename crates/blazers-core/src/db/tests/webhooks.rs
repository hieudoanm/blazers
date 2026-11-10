use crate::db::*;
use crate::models::*;
use rusqlite::Connection;

#[test]
fn test_webhook_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let wh = insert_webhook(
        &conn,
        "wh1",
        "test hook",
        "http://example.com",
        &["event.a".to_string()],
        "sec123",
    )
    .unwrap();
    assert_eq!(wh.name, "test hook");
    assert!(wh.is_active);

    let hooks = list_webhooks(&conn).unwrap();
    assert_eq!(hooks.len(), 1);

    let fetched = get_webhook(&conn, "wh1").unwrap().unwrap();
    assert_eq!(fetched.url, "http://example.com");

    let updated = update_webhook(
        &conn,
        "wh1",
        "updated hook",
        "http://example.net",
        &["event.b".to_string()],
        "newsec",
        false,
    )
    .unwrap();
    assert!(!updated.is_active);
    assert_eq!(updated.name, "updated hook");

    delete_webhook(&conn, "wh1").unwrap();
    assert!(get_webhook(&conn, "wh1").unwrap().is_none());
    assert!(list_webhooks(&conn).unwrap().is_empty());
}

// --- Webhook Log ---

#[test]
fn test_webhook_log_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let log = WebhookLog {
        id: "log1".into(),
        webhook_id: "wh1".into(),
        event: "event.a".into(),
        url: "http://example.com".into(),
        request_body: "{}".into(),
        response_status: 200,
        response_body: "ok".into(),
        error: String::new(),
        status: "delivered".into(),
        created_at: "2024-01-01T00:00:00Z".into(),
    };
    insert_webhook_log(&conn, &log).unwrap();

    let logs = list_webhook_logs(&conn, "wh1").unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].event, "event.a");
}

// --- WS Connection CRUD ---

#[test]
fn test_get_webhook_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(get_webhook(&conn, "nonexistent").unwrap().is_none());
}

#[test]
fn test_list_webhooks_empty() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(list_webhooks(&conn).unwrap().is_empty());
}

#[test]
fn test_list_webhook_logs_empty() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(list_webhook_logs(&conn, "nonexistent").unwrap().is_empty());
}
