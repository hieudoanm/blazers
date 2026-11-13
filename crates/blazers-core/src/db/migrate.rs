//! Schema migrations — all `CREATE TABLE IF NOT EXISTS` + column backfills.

use rusqlite::Connection;

use super::schema::ensure_column;
use crate::models::*;

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
        CREATE TABLE IF NOT EXISTS _functions (
            name       TEXT PRIMARY KEY,
            code       TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
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
        CREATE TABLE IF NOT EXISTS _settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
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

    for (table, column, decl) in [
        ("_collections", "list_rule", "TEXT NOT NULL DEFAULT ''"),
        ("_collections", "view_rule", "TEXT NOT NULL DEFAULT ''"),
        ("_collections", "create_rule", "TEXT NOT NULL DEFAULT ''"),
        ("_collections", "update_rule", "TEXT NOT NULL DEFAULT ''"),
        ("_collections", "delete_rule", "TEXT NOT NULL DEFAULT ''"),
        ("_collections", "type", "TEXT NOT NULL DEFAULT 'records'"),
        ("_users", "data", "TEXT NOT NULL DEFAULT '{}'"),
        ("_users", "verified", "INTEGER NOT NULL DEFAULT 0"),
        ("_users", "reset_token", "TEXT"),
        ("_users", "reset_expires", "TEXT"),
        ("_users", "verify_token", "TEXT"),
        ("_users", "verify_expires", "TEXT"),
        ("_users", "otp_code", "TEXT"),
        ("_users", "otp_expires", "TEXT"),
        ("_users", "mfa_secret", "TEXT"),
        ("_users", "mfa_enabled", "INTEGER NOT NULL DEFAULT 0"),
    ] {
        ensure_column(conn, table, column, decl)?;
    }

    Ok(())
}
