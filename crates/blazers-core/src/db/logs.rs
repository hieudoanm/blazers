use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn insert_log(
    conn: &Connection,
    id: &str,
    level: &str,
    message: &str,
    meta: &str,
) -> Result<LogEntry> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _logs (id, level, message, meta, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, level, message, meta, now],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(LogEntry {
        id: id.to_string(),
        level: level.to_string(),
        message: message.to_string(),
        meta: meta.to_string(),
        created_at: now,
    })
}

pub fn list_logs(conn: &Connection) -> Result<Vec<LogEntry>> {
    let mut stmt = conn
        .prepare("SELECT id, level, message, meta, created_at FROM _logs ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LogEntry {
                id: row.get(0)?,
                level: row.get(1)?,
                message: row.get(2)?,
                meta: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(logs)
}

pub fn clear_logs(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM _logs", [])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
