use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn list_secrets(conn: &Connection) -> Result<Vec<Secret>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, value, scope, created_at, updated_at FROM _secrets ORDER BY name",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![], |row| {
            Ok(Secret {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                scope: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut secrets = Vec::new();
    for row in rows {
        secrets.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(secrets)
}

pub fn get_secret(conn: &Connection, id: &str) -> Result<Option<Secret>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, value, scope, created_at, updated_at FROM _secrets WHERE id = ?1",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(Secret {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                scope: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(s)) => Ok(Some(s)),
        _ => Ok(None),
    }
}

pub fn insert_secret(
    conn: &Connection,
    id: &str,
    name: &str,
    value: &str,
    scope: &str,
) -> Result<Secret> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _secrets (id, name, value, scope, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, name, value, scope, now, now],
    )
    .map_err(|e| AppError::Internal(format!("insert secret: {e}")))?;
    Ok(Secret {
        id: id.to_string(),
        name: name.to_string(),
        value: value.to_string(),
        scope: scope.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_secret(
    conn: &Connection,
    id: &str,
    name: &str,
    value: &str,
    scope: &str,
) -> Result<Secret> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE _secrets SET name = ?1, value = ?2, scope = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, value, scope, now, id],
    )
    .map_err(|e| AppError::Internal(format!("update secret: {e}")))?;
    Ok(Secret {
        id: id.to_string(),
        name: name.to_string(),
        value: value.to_string(),
        scope: scope.to_string(),
        created_at: String::new(),
        updated_at: now,
    })
}

pub fn delete_secret(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM _secrets WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
