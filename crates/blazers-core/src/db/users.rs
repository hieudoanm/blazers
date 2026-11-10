use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

#[allow(clippy::type_complexity)]
pub fn find_user_by_email(
    conn: &Connection,
    email: &str,
) -> Result<Option<(String, String, String, String, String)>> {
    let mut stmt = conn
        .prepare("SELECT id, email, created_at, updated_at, password FROM _users WHERE email = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![email], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(r)) => Ok(Some(r)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn insert_user(conn: &Connection, id: &str, email: &str, password: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _users (id, email, password, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, email, password, now, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("email already registered".into())
        } else {
            AppError::Internal(format!("insert user: {e}"))
        }
    })?;
    Ok(())
}
