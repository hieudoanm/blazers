use crate::db::*;
use rusqlite::Connection;

#[test]
fn test_log_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let log = insert_log(&conn, "log1", "info", "server started", "{}").unwrap();
    assert_eq!(log.level, "info");
    assert_eq!(log.message, "server started");

    let logs = list_logs(&conn).unwrap();
    assert_eq!(logs.len(), 1);

    clear_logs(&conn).unwrap();
    assert!(list_logs(&conn).unwrap().is_empty());
}
