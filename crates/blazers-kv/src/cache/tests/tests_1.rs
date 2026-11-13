use super::*;

#[test]
fn new_creates_empty_cache() {
    let cache = CacheStore::new();
    assert!(cache.data.read().unwrap().is_empty());
}

#[test]
fn list_returns_empty_for_new_cache() {
    let cache = CacheStore::new();
    assert!(cache.list().is_empty());
}

#[test]
fn stats_returns_zero_counts_for_new_cache() {
    let cache = CacheStore::new();
    let stats = cache.stats();
    assert_eq!(stats["total_entries"], 0);
    assert_eq!(stats["expired_entries"], 0);
}

#[test]
fn list_filters_expired_entries() {
    let cache = CacheStore::new();
    let now = Utc::now();
    let past = now - Duration::hours(1);
    let future = now + Duration::hours(1);

    {
        let mut data = cache.data.write().unwrap();
        data.insert(
            "expired".to_string(),
            CacheEntry {
                key: "expired".to_string(),
                value: "old".to_string(),
                ttl: 3600,
                expires_at: past.to_rfc3339(),
                created_at: past.to_rfc3339(),
                updated_at: past.to_rfc3339(),
            },
        );
        data.insert(
            "valid".to_string(),
            CacheEntry {
                key: "valid".to_string(),
                value: "fresh".to_string(),
                ttl: 3600,
                expires_at: future.to_rfc3339(),
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            },
        );
    }

    let entries = cache.list();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "valid");
}

#[test]
fn evict_expired_removes_expired_entries() {
    let cache = CacheStore::new();
    let now = Utc::now();
    let past = now - Duration::hours(1);
    let future = now + Duration::hours(1);

    {
        let mut data = cache.data.write().unwrap();
        data.insert(
            "expired".to_string(),
            CacheEntry {
                key: "expired".to_string(),
                value: "old".to_string(),
                ttl: 3600,
                expires_at: past.to_rfc3339(),
                created_at: past.to_rfc3339(),
                updated_at: past.to_rfc3339(),
            },
        );
        data.insert(
            "valid".to_string(),
            CacheEntry {
                key: "valid".to_string(),
                value: "fresh".to_string(),
                ttl: 3600,
                expires_at: future.to_rfc3339(),
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            },
        );
    }

    cache.evict_expired();
    assert_eq!(cache.data.read().unwrap().len(), 1);
    assert!(cache.data.read().unwrap().contains_key("valid"));
}

#[test]
fn evict_expired_no_expired_does_nothing() {
    let cache = CacheStore::new();
    let now = Utc::now();
    let future = now + Duration::hours(1);

    {
        let mut data = cache.data.write().unwrap();
        data.insert(
            "valid".to_string(),
            CacheEntry {
                key: "valid".to_string(),
                value: "fresh".to_string(),
                ttl: 3600,
                expires_at: future.to_rfc3339(),
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            },
        );
    }

    cache.evict_expired();
    assert_eq!(cache.data.read().unwrap().len(), 1);
}

#[test]
fn stats_counts_expired_and_total_entries() {
    let cache = CacheStore::new();
    let now = Utc::now();
    let past = now - Duration::hours(1);
    let future = now + Duration::hours(1);

    {
        let mut data = cache.data.write().unwrap();
        data.insert(
            "expired".to_string(),
            CacheEntry {
                key: "expired".to_string(),
                value: "old".to_string(),
                ttl: 3600,
                expires_at: past.to_rfc3339(),
                created_at: past.to_rfc3339(),
                updated_at: past.to_rfc3339(),
            },
        );
        data.insert(
            "valid".to_string(),
            CacheEntry {
                key: "valid".to_string(),
                value: "fresh".to_string(),
                ttl: 3600,
                expires_at: future.to_rfc3339(),
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            },
        );
    }

    let stats = cache.stats();
    assert_eq!(stats["total_entries"], 2);
    assert_eq!(stats["expired_entries"], 1);
}

#[test]
fn incr_creates_and_increments() {
    let cache = CacheStore::new();
    let conn = test_conn();
    assert_eq!(cache.incr(&conn, "counter", 1).unwrap(), 1);
    assert_eq!(cache.incr(&conn, "counter", 41).unwrap(), 42);
    assert_eq!(cache.incr(&conn, "counter", -2).unwrap(), 40);
    assert_eq!(cache.get(&conn, "counter").unwrap().unwrap().value, "40");
}

#[test]
fn incr_rejects_non_integer() {
    let cache = CacheStore::new();
    let conn = test_conn();
    cache.set(&conn, "s", "hello", 0).unwrap();
    assert!(cache.incr(&conn, "s", 1).is_err());
    cache
        .list_push(&conn, "l", vec![serde_json::json!(1)], false)
        .unwrap();
    assert!(cache.incr(&conn, "l", 1).is_err());
}

#[test]
fn append_builds_string() {
    let cache = CacheStore::new();
    let conn = test_conn();
    assert_eq!(cache.append(&conn, "log", "a").unwrap(), "a");
    assert_eq!(cache.append(&conn, "log", "b").unwrap(), "ab");
    assert_eq!(cache.get(&conn, "log").unwrap().unwrap().value, "ab");
}
