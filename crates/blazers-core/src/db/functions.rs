use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

const FN_COLS: &str = "name, code, created_at, updated_at";

fn row_to_function(row: &rusqlite::Row) -> rusqlite::Result<Function> {
    Ok(Function {
        name: row.get(0)?,
        code: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

/// Creates or replaces a function by name.
pub fn upsert_function(conn: &Connection, name: &str, code: &str) -> Result<Function> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _functions (name, code, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)
         ON CONFLICT(name) DO UPDATE SET code = ?2, updated_at = ?3",
        params![name, code, now],
    )
    .map_err(|e| AppError::Internal(format!("upsert function: {e}")))?;
    get_function(conn, name)?
        .ok_or_else(|| AppError::Internal("function not found after upsert".into()))
}

pub fn get_function(conn: &Connection, name: &str) -> Result<Option<Function>> {
    let mut stmt = conn
        .prepare(&format!("SELECT {FN_COLS} FROM _functions WHERE name = ?1"))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![name], row_to_function)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(f)) => Ok(Some(f)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_functions(conn: &Connection) -> Result<Vec<Function>> {
    let mut stmt = conn
        .prepare(&format!("SELECT {FN_COLS} FROM _functions ORDER BY name"))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], row_to_function)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn delete_function(conn: &Connection, name: &str) -> Result<bool> {
    let n = conn
        .execute("DELETE FROM _functions WHERE name = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(n > 0)
}
