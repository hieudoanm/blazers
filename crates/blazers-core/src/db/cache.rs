use rusqlite::{Connection, params};

use crate::models::*;

pub fn set_cache_entry(conn: &Connection, key: &str, value: &str, ttl: i64) -> Result<CacheEntry> {
    let now = chrono::Utc::now().to_rfc3339();
    let expires_at = if ttl > 0 {
        (chrono::Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339()
    } else {
        String::new()
    };
    conn.execute(
        "INSERT INTO _cache (key, value, ttl, expires_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, ttl = excluded.ttl, expires_at = excluded.expires_at, updated_at = excluded.updated_at",
        params![key, value, ttl, expires_at, now, now],
    )
    .map_err(|e| AppError::Internal(format!("set cache: {e}")))?;
    Ok(CacheEntry {
        key: key.to_string(),
        value: value.to_string(),
        ttl,
        expires_at,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_cache_entry(conn: &Connection, key: &str) -> Result<Option<CacheEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT key, value, ttl, expires_at, created_at, updated_at FROM _cache WHERE key = ?1",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![key], |row| {
            Ok(CacheEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                ttl: row.get(2)?,
                expires_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(entry)) => {
            if entry.ttl > 0
                && !entry.expires_at.is_empty()
                && let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&entry.expires_at)
                && chrono::Utc::now() > expires
            {
                conn.execute("DELETE FROM _cache WHERE key = ?1", params![key])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                return Ok(None);
            }
            Ok(Some(entry))
        }
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn delete_cache_entry(conn: &Connection, key: &str) -> Result<bool> {
    let n = conn
        .execute("DELETE FROM _cache WHERE key = ?1", params![key])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(n > 0)
}

pub fn list_cache_entries(conn: &Connection) -> Result<Vec<CacheEntry>> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut stmt = conn
        .prepare("SELECT key, value, ttl, expires_at, created_at, updated_at FROM _cache WHERE ttl = 0 OR expires_at > ?1 ORDER BY updated_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![now], |row| {
            Ok(CacheEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                ttl: row.get(2)?,
                expires_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(entries)
}

pub fn flush_cache(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM _cache", [])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

/// Update only the value of an existing entry, preserving ttl/expires_at.
pub fn update_cache_value(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "UPDATE _cache SET value = ?2, updated_at = ?3 WHERE key = ?1",
        params![key, value, chrono::Utc::now().to_rfc3339()],
    )
    .map_err(|e| AppError::Internal(format!("update cache: {e}")))?;
    Ok(())
}

/// Set a new TTL on an existing entry without touching its value.
/// `ttl <= 0` clears the expiry. Returns false when the key is missing.
pub fn expire_cache_entry(conn: &Connection, key: &str, ttl: i64) -> Result<bool> {
    let expires_at = if ttl > 0 {
        (chrono::Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339()
    } else {
        String::new()
    };
    let n = conn
        .execute(
            "UPDATE _cache SET ttl = ?2, expires_at = ?3, updated_at = ?4 WHERE key = ?1",
            params![key, ttl, expires_at, chrono::Utc::now().to_rfc3339()],
        )
        .map_err(|e| AppError::Internal(format!("expire cache: {e}")))?;
    Ok(n > 0)
}
