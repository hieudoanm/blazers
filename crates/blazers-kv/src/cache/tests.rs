use super::*;
use chrono::{Duration, Utc};

fn test_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::migrate_db(&conn).unwrap();
    conn
}

mod tests_1;
mod tests_2;
