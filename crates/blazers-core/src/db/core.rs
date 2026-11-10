use std::path::PathBuf;

use rusqlite::Connection;

use crate::models::*;

pub fn data_dir() -> PathBuf {
    let dir = std::env::var("BLAZERS_DATA").unwrap_or_default();
    if dir.is_empty() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".blazers")
    } else {
        PathBuf::from(dir)
    }
}

pub struct ConnectionManager {
    pub path: PathBuf,
}

impl deadpool::managed::Manager for ConnectionManager {
    type Type = rusqlite::Connection;
    type Error = rusqlite::Error;

    async fn create(&self) -> std::result::Result<Self::Type, Self::Error> {
        rusqlite::Connection::open(&self.path)
    }

    async fn recycle(
        &self,
        _: &mut Self::Type,
        _: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

pub fn migrate_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _collections (
            name       TEXT PRIMARY KEY,
            schema     TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _users (
            id         TEXT PRIMARY KEY,
            email      TEXT UNIQUE NOT NULL,
            password   TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _buckets (
            name       TEXT PRIMARY KEY,
            is_public  INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _files (
            id         TEXT PRIMARY KEY,
            bucket     TEXT NOT NULL,
            filename   TEXT NOT NULL,
            mime_type  TEXT NOT NULL DEFAULT 'application/octet-stream',
            size       INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _webhooks (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            url          TEXT NOT NULL,
            events       TEXT NOT NULL DEFAULT '[]',
            secret       TEXT NOT NULL DEFAULT '',
            is_active    INTEGER NOT NULL DEFAULT 1,
            created_at   TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _webhook_logs (
            id              TEXT PRIMARY KEY,
            webhook_id      TEXT NOT NULL,
            event           TEXT NOT NULL,
            url             TEXT NOT NULL,
            request_body    TEXT NOT NULL DEFAULT '',
            response_status INTEGER NOT NULL DEFAULT 0,
            response_body   TEXT NOT NULL DEFAULT '',
            error           TEXT NOT NULL DEFAULT '',
            status          TEXT NOT NULL DEFAULT '',
            created_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _secrets (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            value        TEXT NOT NULL,
            scope        TEXT NOT NULL DEFAULT 'general',
            created_at   TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _ws_connections (
            id              TEXT PRIMARY KEY,
            remote_addr     TEXT NOT NULL,
            path            TEXT NOT NULL DEFAULT '/',
            user_agent      TEXT NOT NULL DEFAULT '',
            connected_at    TEXT NOT NULL,
            disconnected_at TEXT NOT NULL DEFAULT '',
            is_active       INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _ws_messages (
            id            TEXT PRIMARY KEY,
            connection_id TEXT,
            direction     TEXT NOT NULL,
            content       TEXT NOT NULL,
            created_at    TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _cache (
            key        TEXT PRIMARY KEY,
            value      TEXT NOT NULL,
            ttl        INTEGER NOT NULL DEFAULT 0,
            expires_at TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| AppError::Internal(format!("migrate: {e}")))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS _notifications (
            id         TEXT PRIMARY KEY,
            title      TEXT NOT NULL,
            body       TEXT NOT NULL DEFAULT '',
            type       TEXT NOT NULL DEFAULT 'info',
            is_read    INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        [],
    )
    .map_err(|e| AppError::Internal(format!("migrate: {e}")))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS _logs (
            id         TEXT PRIMARY KEY,
            level      TEXT NOT NULL DEFAULT 'info',
            message    TEXT NOT NULL,
            meta       TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        [],
    )
    .map_err(|e| AppError::Internal(format!("migrate: {e}")))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS _pubsub_topics (
            id         TEXT PRIMARY KEY,
            name       TEXT UNIQUE NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        [],
    )
    .map_err(|e| AppError::Internal(format!("migrate: {e}")))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS _pubsub_messages (
            id         TEXT PRIMARY KEY,
            topic_id   TEXT NOT NULL,
            body       TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES _pubsub_topics(id) ON DELETE CASCADE
        );",
        [],
    )
    .map_err(|e| AppError::Internal(format!("migrate: {e}")))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _permissions (
            id         TEXT PRIMARY KEY,
            user_id    TEXT NOT NULL,
            collection TEXT NOT NULL,
            role       TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(user_id, collection)
        );",
    )
    .map_err(|e| AppError::Internal(format!("migrate: {e}")))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _cronjobs (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            schedule        TEXT NOT NULL,
            command         TEXT NOT NULL,
            method          TEXT NOT NULL DEFAULT 'GET',
            headers         TEXT NOT NULL DEFAULT '',
            body            TEXT NOT NULL DEFAULT '',
            is_active       INTEGER NOT NULL DEFAULT 1,
            last_run_at     TEXT NOT NULL DEFAULT '',
            last_run_status TEXT NOT NULL DEFAULT '',
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS _cronjob_logs (
            id          TEXT PRIMARY KEY,
            cronjob_id  TEXT NOT NULL,
            started_at  TEXT NOT NULL,
            finished_at TEXT NOT NULL,
            duration_ms INTEGER NOT NULL DEFAULT 0,
            status      TEXT NOT NULL DEFAULT '',
            output      TEXT NOT NULL DEFAULT '',
            error       TEXT NOT NULL DEFAULT ''
        );",
    )
    .map_err(|e| AppError::Internal(format!("migrate: {e}")))?;

    Ok(())
}
