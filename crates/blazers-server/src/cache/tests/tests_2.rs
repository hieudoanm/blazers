use super::*;

#[test]
fn expire_and_ttl_roundtrip() {
    let cache = CacheStore::new();
    let conn = test_conn();
    cache.set(&conn, "k", "v", 0).unwrap();
    assert_eq!(cache.ttl(&conn, "k").unwrap(), Some(-1));
    assert!(cache.expire(&conn, "k", 100).unwrap().is_some());
    let ttl = cache.ttl(&conn, "k").unwrap().unwrap();
    assert!((90..=100).contains(&ttl));
    assert!(cache.expire(&conn, "k", 0).unwrap().is_some());
    assert_eq!(cache.ttl(&conn, "k").unwrap(), Some(-1));
    assert!(cache.expire(&conn, "missing", 10).unwrap().is_none());
    assert_eq!(cache.ttl(&conn, "missing").unwrap(), None);
}

#[test]
fn list_push_pop_range() {
    let cache = CacheStore::new();
    let conn = test_conn();
    assert_eq!(
        cache
            .list_push(
                &conn,
                "q",
                vec![serde_json::json!("a"), serde_json::json!("b")],
                false,
            )
            .unwrap(),
        2
    );
    cache
        .list_push(&conn, "q", vec![serde_json::json!("c")], true)
        .unwrap();
    assert_eq!(cache.list_len(&conn, "q").unwrap(), 3);
    let range = cache.list_range(&conn, "q", 0, -1).unwrap();
    assert_eq!(
        range,
        vec![
            serde_json::json!("c"),
            serde_json::json!("a"),
            serde_json::json!("b")
        ]
    );
    assert_eq!(
        cache.list_pop(&conn, "q", true, None).unwrap(),
        Some(serde_json::json!("c"))
    );
    assert_eq!(
        cache.list_pop(&conn, "q", false, Some(2)).unwrap(),
        Some(serde_json::json!(["a", "b"]))
    );
    assert_eq!(cache.list_pop(&conn, "q", true, None).unwrap(), None);
    assert!(cache.get(&conn, "q").unwrap().is_none());
}

#[test]
fn list_rejects_string_key() {
    let cache = CacheStore::new();
    let conn = test_conn();
    cache.set(&conn, "s", "v", 0).unwrap();
    assert!(
        cache
            .list_push(&conn, "s", vec![serde_json::json!(1)], false)
            .is_err()
    );
    assert!(cache.list_range(&conn, "s", 0, -1).is_err());
}

#[test]
fn hash_set_get_del() {
    let cache = CacheStore::new();
    let conn = test_conn();
    let mut fields = serde_json::Map::new();
    fields.insert("name".into(), serde_json::json!("ada"));
    fields.insert("age".into(), serde_json::json!(36));
    assert_eq!(cache.hash_set(&conn, "user:1", fields).unwrap(), 2);
    assert_eq!(cache.hash_len(&conn, "user:1").unwrap(), 2);
    assert_eq!(
        cache.hash_get(&conn, "user:1", "name").unwrap(),
        Some(serde_json::json!("ada"))
    );
    let all = cache.hash_get_all(&conn, "user:1").unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(cache.hash_del(&conn, "user:1", &["age".into()]).unwrap(), 1);
    assert_eq!(cache.hash_len(&conn, "user:1").unwrap(), 1);
    assert_eq!(
        cache.hash_del(&conn, "user:1", &["name".into()]).unwrap(),
        1
    );
    assert!(cache.get(&conn, "user:1").unwrap().is_none());
}

#[test]
fn hash_rejects_string_key() {
    let cache = CacheStore::new();
    let conn = test_conn();
    cache.set(&conn, "s", "v", 0).unwrap();
    let mut fields = serde_json::Map::new();
    fields.insert("f".into(), serde_json::json!(1));
    assert!(cache.hash_set(&conn, "s", fields).is_err());
}

#[test]
fn typed_values_survive_plain_get() {
    let cache = CacheStore::new();
    let conn = test_conn();
    cache
        .list_push(&conn, "l", vec![serde_json::json!("x")], false)
        .unwrap();
    let raw = cache.get(&conn, "l").unwrap().unwrap().value;
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["__type"], "list");
    let mut fields = serde_json::Map::new();
    fields.insert("a".into(), serde_json::json!(1));
    cache.hash_set(&conn, "h", fields).unwrap();
    let raw = cache.get(&conn, "h").unwrap().unwrap().value;
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["__type"], "hash");
}

#[test]
fn ttl_survives_value_update() {
    let cache = CacheStore::new();
    let conn = test_conn();
    cache.set(&conn, "k", "1", 0).unwrap();
    cache.expire(&conn, "k", 500).unwrap();
    cache.incr(&conn, "k", 1).unwrap();
    let ttl = cache.ttl(&conn, "k").unwrap().unwrap();
    assert!((400..=500).contains(&ttl));
}
