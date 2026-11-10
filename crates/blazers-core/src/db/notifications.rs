use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn insert_notification(
    conn: &Connection,
    id: &str,
    title: &str,
    body: &str,
    ntype: &str,
) -> Result<Notification> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _notifications (id, title, body, type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, title, body, ntype, now],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Notification {
        id: id.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        ntype: ntype.to_string(),
        is_read: false,
        created_at: now,
    })
}

pub fn list_notifications(conn: &Connection) -> Result<Vec<Notification>> {
    let mut stmt = conn
        .prepare("SELECT id, title, body, type, is_read, created_at FROM _notifications ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            let is_read_int: i64 = row.get(4)?;
            Ok(Notification {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                ntype: row.get(3)?,
                is_read: is_read_int != 0,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut notifications = Vec::new();
    for row in rows {
        notifications.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(notifications)
}

pub fn get_notification(conn: &Connection, id: &str) -> Result<Option<Notification>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, body, type, is_read, created_at FROM _notifications WHERE id = ?1",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            let is_read_int: i64 = row.get(4)?;
            Ok(Notification {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                ntype: row.get(3)?,
                is_read: is_read_int != 0,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(notification)) => Ok(Some(notification)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn update_notification_read(conn: &Connection, id: &str) -> Result<Notification> {
    conn.execute(
        "UPDATE _notifications SET is_read = 1 WHERE id = ?1",
        params![id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    get_notification(conn, id)?.ok_or_else(|| AppError::NotFound("not found".into()))
}

pub fn delete_notification(conn: &Connection, id: &str) -> Result<bool> {
    let affected = conn
        .execute("DELETE FROM _notifications WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(affected > 0)
}

pub fn clear_notifications(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM _notifications", [])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
