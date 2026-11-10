use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn list_webhooks(conn: &Connection) -> Result<Vec<Webhook>> {
    let mut stmt = conn
        .prepare("SELECT id, name, url, events, secret, is_active, created_at, updated_at FROM _webhooks ORDER BY name")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            let events_str: String = row.get(3)?;
            let events: Vec<String> = serde_json::from_str(&events_str).unwrap_or_default();
            Ok(Webhook {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                events,
                secret: row.get(4)?,
                is_active: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut hooks = Vec::new();
    for row in rows {
        hooks.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(hooks)
}

pub fn get_webhook(conn: &Connection, id: &str) -> Result<Option<Webhook>> {
    let mut stmt = conn
        .prepare("SELECT id, name, url, events, secret, is_active, created_at, updated_at FROM _webhooks WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            let events_str: String = row.get(3)?;
            let events: Vec<String> = serde_json::from_str(&events_str).unwrap_or_default();
            Ok(Webhook {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                events,
                secret: row.get(4)?,
                is_active: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(h)) => Ok(Some(h)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn insert_webhook(
    conn: &Connection,
    id: &str,
    name: &str,
    url: &str,
    events: &[String],
    secret: &str,
) -> Result<Webhook> {
    let now = Utc::now().to_rfc3339();
    let events_str = serde_json::to_string(events).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO _webhooks (id, name, url, events, secret, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, url, events_str, secret, now, now],
    )
    .map_err(|e| AppError::Internal(format!("insert webhook: {e}")))?;
    Ok(Webhook {
        id: id.to_string(),
        name: name.to_string(),
        url: url.to_string(),
        events: events.to_vec(),
        secret: secret.to_string(),
        is_active: true,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_webhook(
    conn: &Connection,
    id: &str,
    name: &str,
    url: &str,
    events: &[String],
    secret: &str,
    is_active: bool,
) -> Result<Webhook> {
    let now = Utc::now().to_rfc3339();
    let events_str = serde_json::to_string(events).unwrap_or_else(|_| "[]".to_string());
    let active = if is_active { 1 } else { 0 };
    conn.execute(
        "UPDATE _webhooks SET name = ?1, url = ?2, events = ?3, secret = ?4, is_active = ?5, updated_at = ?6 WHERE id = ?7",
        params![name, url, events_str, secret, active, now, id],
    )
    .map_err(|e| AppError::Internal(format!("update webhook: {e}")))?;
    Ok(Webhook {
        id: id.to_string(),
        name: name.to_string(),
        url: url.to_string(),
        events: events.to_vec(),
        secret: secret.to_string(),
        is_active,
        created_at: String::new(),
        updated_at: now,
    })
}

pub fn delete_webhook(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM _webhook_logs WHERE webhook_id = ?1",
        params![id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM _webhooks WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn list_webhook_logs(conn: &Connection, webhook_id: &str) -> Result<Vec<WebhookLog>> {
    let mut stmt = conn
        .prepare("SELECT id, webhook_id, event, url, request_body, response_status, response_body, error, status, created_at FROM _webhook_logs WHERE webhook_id = ?1 ORDER BY created_at DESC LIMIT 50")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![webhook_id], |row| {
            Ok(WebhookLog {
                id: row.get(0)?,
                webhook_id: row.get(1)?,
                event: row.get(2)?,
                url: row.get(3)?,
                request_body: row.get(4)?,
                response_status: row.get(5)?,
                response_body: row.get(6)?,
                error: row.get(7)?,
                status: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(logs)
}

pub fn insert_webhook_log(conn: &Connection, log: &WebhookLog) -> Result<()> {
    conn.execute(
        "INSERT INTO _webhook_logs (id, webhook_id, event, url, request_body, response_status, response_body, error, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![log.id, log.webhook_id, log.event, log.url, log.request_body, log.response_status, log.response_body, log.error, log.status, log.created_at],
    )
    .map_err(|e| AppError::Internal(format!("insert webhook log: {e}")))?;
    Ok(())
}
