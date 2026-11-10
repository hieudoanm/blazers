use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn insert_ws_connection(
    conn: &Connection,
    id: &str,
    remote_addr: &str,
    path: &str,
    user_agent: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _ws_connections (id, remote_addr, path, user_agent, connected_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, remote_addr, path, user_agent, now],
    )
    .map_err(|e| AppError::Internal(format!("insert ws connection: {e}")))?;
    Ok(())
}

pub fn update_ws_disconnect(conn: &Connection, id: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE _ws_connections SET disconnected_at = ?1, is_active = 0 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| AppError::Internal(format!("update ws disconnect: {e}")))?;
    Ok(())
}

pub fn list_ws_connections(conn: &Connection) -> Result<Vec<WSConnection>> {
    let mut stmt = conn
        .prepare("SELECT id, remote_addr, path, user_agent, connected_at, disconnected_at, is_active, created_at FROM _ws_connections ORDER BY connected_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(WSConnection {
                id: row.get(0)?,
                remote_addr: row.get(1)?,
                path: row.get(2)?,
                user_agent: row.get(3)?,
                connected_at: row.get(4)?,
                disconnected_at: row.get(5)?,
                is_active: row.get::<_, i64>(6)? != 0,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut connections = Vec::new();
    for row in rows {
        connections.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(connections)
}

pub fn get_ws_connection(conn: &Connection, id: &str) -> Result<Option<WSConnection>> {
    let mut stmt = conn
        .prepare("SELECT id, remote_addr, path, user_agent, connected_at, disconnected_at, is_active, created_at FROM _ws_connections WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(WSConnection {
                id: row.get(0)?,
                remote_addr: row.get(1)?,
                path: row.get(2)?,
                user_agent: row.get(3)?,
                connected_at: row.get(4)?,
                disconnected_at: row.get(5)?,
                is_active: row.get::<_, i64>(6)? != 0,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(c)) => Ok(Some(c)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn delete_ws_connection(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM _ws_messages WHERE connection_id = ?1",
        params![id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM _ws_connections WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn insert_ws_message(
    conn: &Connection,
    connection_id: &str,
    direction: &str,
    content: &str,
) -> Result<()> {
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    conn.execute(
        "INSERT INTO _ws_messages (id, connection_id, direction, content) VALUES (?1, ?2, ?3, ?4)",
        params![id, connection_id, direction, content],
    )
    .map_err(|e| AppError::Internal(format!("insert ws message: {e}")))?;
    Ok(())
}

pub fn list_ws_messages(conn: &Connection, connection_id: &str) -> Result<Vec<WSMessage>> {
    let mut stmt = conn
        .prepare("SELECT id, connection_id, direction, content, created_at FROM _ws_messages WHERE connection_id = ?1 ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![connection_id], |row| {
            Ok(WSMessage {
                id: row.get(0)?,
                connection_id: row.get(1)?,
                direction: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(messages)
}

pub fn list_all_ws_messages(conn: &Connection) -> Result<Vec<WSMessage>> {
    let mut stmt = conn
        .prepare("SELECT id, connection_id, direction, content, created_at FROM _ws_messages ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(WSMessage {
                id: row.get(0)?,
                connection_id: row.get(1)?,
                direction: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(messages)
}
