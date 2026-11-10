use super::*;

fn setup() -> (CacheStore, rusqlite::Connection) {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    blazers_core::db::migrate_db(&conn).unwrap();
    (CacheStore::new(), conn)
}

fn call(line: &str, cache: &CacheStore, conn: &rusqlite::Connection) -> String {
    let mut authed = false;
    handle_line(line, cache, conn, &mut authed, false).0
}

#[test]
fn ping_and_empty_lines() {
    let (cache, conn) = setup();
    assert_eq!(call("PING", &cache, &conn), "PONG\n");
    assert_eq!(call("ping", &cache, &conn), "PONG\n");
    assert_eq!(call("   \r\n", &cache, &conn), "");
    assert_eq!(
        call("NOSUCH", &cache, &conn),
        "ERR unknown command 'NOSUCH'\n"
    );
}

#[test]
fn set_get_del_ex() {
    let (cache, conn) = setup();
    assert_eq!(call("SET k hello world", &cache, &conn), "OK\n");
    assert_eq!(call("GET k", &cache, &conn), "hello world\n");
    assert_eq!(call("GET missing", &cache, &conn), "(nil)\n");
    assert_eq!(call("SET k v EX 30", &cache, &conn), "OK\n");
    let ttl = call("TTL k", &cache, &conn);
    assert!(ttl.starts_with("2") || ttl.starts_with("3"), "ttl={ttl}");
    assert_eq!(call("EXISTS k", &cache, &conn), "1\n");
    assert_eq!(call("DEL k missing", &cache, &conn), "1\n");
    assert_eq!(call("EXISTS k", &cache, &conn), "0\n");
}

#[test]
fn keys_len_type_flush() {
    let (cache, conn) = setup();
    call("SET a 1", &cache, &conn);
    call("SET b 2", &cache, &conn);
    call("RPUSH l x", &cache, &conn);
    assert_eq!(call("LEN", &cache, &conn), "3\n");
    assert_eq!(call("KEYS *", &cache, &conn), "a b l\n");
    assert_eq!(call("KEYS l*", &cache, &conn), "l\n");
    assert_eq!(call("TYPE l", &cache, &conn), "list\n");
    assert_eq!(call("TYPE missing", &cache, &conn), "none\n");
    assert_eq!(call("FLUSHALL", &cache, &conn), "OK\n");
    assert_eq!(call("LEN", &cache, &conn), "0\n");
}

#[test]
fn incr_append_typed_ops() {
    let (cache, conn) = setup();
    assert_eq!(call("INCR n", &cache, &conn), "1\n");
    assert_eq!(call("INCRBY n 41", &cache, &conn), "42\n");
    assert_eq!(call("DECR n", &cache, &conn), "41\n");
    assert_eq!(call("APPEND s foo", &cache, &conn), "3\n");
    assert_eq!(call("APPEND s bar", &cache, &conn), "6\n");
    assert_eq!(call("GET s", &cache, &conn), "foobar\n");
    assert_eq!(
        call("INCR s", &cache, &conn),
        "ERR WRONGTYPE Operation against a string key\n"
    );

    call("RPUSH q a", &cache, &conn);
    call("RPUSH q b c", &cache, &conn);
    assert_eq!(call("LRANGE q 0 -1", &cache, &conn), "a b c\n");
    assert_eq!(call("LLEN", &cache, &conn), "ERR unknown command 'LLEN'\n");
    assert_eq!(call("LPOP q", &cache, &conn), "a\n");
    assert_eq!(call("LRANGE q 0 -1", &cache, &conn), "b c\n");

    assert_eq!(call("HSET u name ada age 36", &cache, &conn), "2\n");
    assert_eq!(call("HGET u name", &cache, &conn), "ada\n");
    assert_eq!(call("HLEN u", &cache, &conn), "2\n");
    assert_eq!(call("HGETALL u", &cache, &conn), "age 36 name ada\n");
    assert_eq!(call("HDEL u age", &cache, &conn), "1\n");
}

#[test]
fn expire_ttl_sentinels() {
    let (cache, conn) = setup();
    assert_eq!(call("EXPIRE missing 10", &cache, &conn), "0\n");
    call("SET k v", &cache, &conn);
    assert_eq!(call("TTL k", &cache, &conn), "-1\n");
    assert_eq!(call("TTL missing", &cache, &conn), "-2\n");
    assert_eq!(call("EXPIRE k 30", &cache, &conn), "1\n");
    let ttl: i64 = call("TTL k", &cache, &conn).trim().parse().unwrap();
    assert!((28..=30).contains(&ttl), "ttl={ttl}");
    assert_eq!(
        call("EXPIRE k 0", &cache, &conn),
        "ERR invalid expire time\n"
    );
}

#[test]
fn wrongtype_errors() {
    let (cache, conn) = setup();
    call("SET s v", &cache, &conn);
    let err = call("RPUSH s x", &cache, &conn);
    assert!(err.starts_with("ERR WRONGTYPE"), "{err}");
    call("RPUSH l x", &cache, &conn);
    let err = call("HSET l f v", &cache, &conn);
    assert!(err.starts_with("ERR WRONGTYPE"), "{err}");
    let err = call("INCR l", &cache, &conn);
    assert!(err.starts_with("ERR WRONGTYPE"), "{err}");
}
