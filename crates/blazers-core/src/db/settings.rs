use rusqlite::{Connection, params};
use serde_json::{Map, Value};

use crate::models::*;

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM _settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .map(Some)
    .or_else(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(AppError::Internal(other.to_string())),
    })
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO _settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| AppError::Internal(format!("set setting: {e}")))?;
    Ok(())
}

pub fn delete_setting(conn: &Connection, key: &str) -> Result<bool> {
    let n = conn
        .execute("DELETE FROM _settings WHERE key = ?1", params![key])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(n > 0)
}

pub fn list_settings(conn: &Connection) -> Result<Map<String, Value>> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM _settings ORDER BY key")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut map = Map::new();
    for row in rows {
        let (k, v) = row.map_err(|e| AppError::Internal(e.to_string()))?;
        map.insert(k, Value::String(v));
    }
    Ok(map)
}
