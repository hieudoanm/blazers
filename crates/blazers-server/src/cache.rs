use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;
use rusqlite::Connection;

use blazers_core::db;
use blazers_core::models::*;

pub struct CacheStore {
    pub data: RwLock<HashMap<String, CacheEntry>>,
}

impl CacheStore {
    pub fn new() -> Self {
        CacheStore {
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_from_db(&self, conn: &Connection) {
        if let Ok(entries) = db::list_cache_entries(conn) {
            let mut data = self.data.write().unwrap();
            for entry in entries {
                data.insert(entry.key.clone(), entry);
            }
        }
    }

    pub fn set(&self, conn: &Connection, key: &str, value: &str, ttl: i64) -> Result<CacheEntry> {
        let entry = db::set_cache_entry(conn, key, value, ttl)?;
        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), entry.clone());
        Ok(entry)
    }

    pub fn get(&self, conn: &Connection, key: &str) -> Result<Option<CacheEntry>> {
        {
            let data = self.data.read().unwrap();
            if let Some(entry) = data.get(key) {
                if entry.ttl > 0
                    && !entry.expires_at.is_empty()
                    && let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&entry.expires_at)
                    && Utc::now() > expires
                {
                    drop(data);
                    let mut data = self.data.write().unwrap();
                    data.remove(key);
                    let _ = db::delete_cache_entry(conn, key);
                    return Ok(None);
                }
                return Ok(Some(entry.clone()));
            }
        }

        if let Some(entry) = db::get_cache_entry(conn, key)? {
            let mut data = self.data.write().unwrap();
            data.insert(key.to_string(), entry.clone());
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, conn: &Connection, key: &str) -> Result<bool> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        db::delete_cache_entry(conn, key)
    }

    pub fn list(&self) -> Vec<CacheEntry> {
        let data = self.data.read().unwrap();
        let now = Utc::now();
        data.values()
            .filter(|e| {
                if e.ttl > 0
                    && !e.expires_at.is_empty()
                    && let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&e.expires_at)
                {
                    return now <= expires;
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn flush(&self, conn: &Connection) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.clear();
        db::flush_cache(conn)
    }

    pub fn stats(&self) -> serde_json::Value {
        let data = self.data.read().unwrap();
        let now = Utc::now();
        let total = data.len();
        let expired = data
            .values()
            .filter(|e| {
                if e.ttl > 0
                    && !e.expires_at.is_empty()
                    && let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&e.expires_at)
                {
                    return now > expires;
                }
                false
            })
            .count();
        serde_json::json!({
            "total_entries": total,
            "expired_entries": expired,
            "memory": true,
        })
    }

    pub fn evict_expired(&self) {
        let mut data = self.data.write().unwrap();
        let now = Utc::now();
        data.retain(|_, e| {
            if e.ttl > 0
                && !e.expires_at.is_empty()
                && let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&e.expires_at)
            {
                return now <= expires;
            }
            true
        });
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn start_eviction_loop(cache: std::sync::Arc<CacheStore>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            cache.evict_expired();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

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
}
