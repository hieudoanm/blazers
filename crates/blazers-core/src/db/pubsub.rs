use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn insert_pubsub_topic(conn: &Connection, id: &str, name: &str) -> Result<PubSubTopic> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _pubsub_topics (id, name, created_at) VALUES (?1, ?2, ?3)",
        params![id, name, now],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(PubSubTopic {
        id: id.to_string(),
        name: name.to_string(),
        created_at: now,
    })
}

pub fn list_pubsub_topics(conn: &Connection) -> Result<Vec<PubSubTopic>> {
    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM _pubsub_topics ORDER BY created_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PubSubTopic {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut topics = Vec::new();
    for row in rows {
        topics.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(topics)
}

pub fn get_pubsub_topic_by_name(conn: &Connection, name: &str) -> Result<Option<PubSubTopic>> {
    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM _pubsub_topics WHERE name = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![name], |row| {
            Ok(PubSubTopic {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(t)) => Ok(Some(t)),
        _ => Ok(None),
    }
}

pub fn delete_pubsub_topic(conn: &Connection, name: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM _pubsub_messages WHERE topic_id = (SELECT id FROM _pubsub_topics WHERE name = ?1)",
        params![name],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM _pubsub_topics WHERE name = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn insert_pubsub_message(
    conn: &Connection,
    id: &str,
    topic_id: &str,
    body: &str,
) -> Result<PubSubMessage> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _pubsub_messages (id, topic_id, body, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, topic_id, body, now],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(PubSubMessage {
        id: id.to_string(),
        topic_id: topic_id.to_string(),
        body: body.to_string(),
        created_at: now,
    })
}

pub fn list_pubsub_messages(conn: &Connection, topic_id: &str) -> Result<Vec<PubSubMessage>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, topic_id, body, created_at FROM _pubsub_messages WHERE topic_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![topic_id], |row| {
            Ok(PubSubMessage {
                id: row.get(0)?,
                topic_id: row.get(1)?,
                body: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut msgs = Vec::new();
    for row in rows {
        msgs.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(msgs)
}
