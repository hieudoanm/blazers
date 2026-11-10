use std::collections::HashMap;
use std::path::PathBuf;

use chrono::Utc;
use rusqlite::{Connection, params};
use serde_json::Value;

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

pub fn create_collection_table(conn: &Connection, name: &str, schema: &str) -> Result<()> {
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS \"_data_{name}\" (
            id         TEXT PRIMARY KEY,
            data       TEXT NOT NULL DEFAULT '{{}}',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    );
    conn.execute(&sql, [])
        .map_err(|e| AppError::Internal(format!("create table: {e}")))?;
    for (field, sql_type) in get_schema_fields(schema) {
        let alter = format!("ALTER TABLE \"_data_{name}\" ADD COLUMN \"{field}\" {sql_type}");
        conn.execute(&alter, [])
            .map_err(|e| AppError::Internal(format!("add column: {e}")))?;
    }
    Ok(())
}

pub fn get_schema_fields(schema: &str) -> Vec<(String, String)> {
    if schema.is_empty() || schema == "{}" {
        return vec![];
    }
    let map: serde_json::Map<String, Value> = serde_json::from_str(schema).unwrap_or_default();
    let mut fields = Vec::new();
    for (field, type_val) in &map {
        let type_str = type_val.as_str().unwrap_or("string");
        let sql_type = match type_str {
            "string" | "email" | "url" => "TEXT",
            "number" => "REAL",
            "integer" => "INTEGER",
            "boolean" => "INTEGER",
            "array" | "object" => "TEXT",
            _ => "TEXT",
        };
        let field_name = if field.ends_with('?') {
            field[..field.len() - 1].to_string()
        } else {
            field.clone()
        };
        fields.push((field_name, sql_type.to_string()));
    }
    fields
}

pub fn migrate_collection_schema(
    conn: &Connection,
    name: &str,
    old_schema: &str,
    new_schema: &str,
) -> Result<()> {
    let old_fields: HashMap<String, String> = get_schema_fields(old_schema).into_iter().collect();
    let new_fields: HashMap<String, String> = get_schema_fields(new_schema).into_iter().collect();
    for (field, sql_type) in &new_fields {
        if !old_fields.contains_key(field) {
            let alter = format!("ALTER TABLE \"_data_{name}\" ADD COLUMN \"{field}\" {sql_type}");
            conn.execute(&alter, [])
                .map_err(|e| AppError::Internal(format!("add column: {e}")))?;
        }
    }
    for field in old_fields.keys() {
        if !new_fields.contains_key(field) {
            let alter = format!("ALTER TABLE \"_data_{name}\" DROP COLUMN \"{field}\"");
            conn.execute(&alter, [])
                .map_err(|e| AppError::Internal(format!("drop column: {e}")))?;
        }
    }
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE _collections SET schema = ?1, updated_at = ?2 WHERE name = ?3",
        params![new_schema, now, name],
    )
    .map_err(|e| AppError::Internal(format!("update collection schema: {e}")))?;
    Ok(())
}

pub fn update_collection(
    conn: &Connection,
    name: &str,
    schema: Option<&str>,
) -> Result<Collection> {
    let existing =
        get_collection(conn, name)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    if let Some(new_schema) = schema
        && !new_schema.is_empty()
        && new_schema != existing.schema
    {
        migrate_collection_schema(conn, name, &existing.schema, new_schema)?;
    }
    get_collection(conn, name)?
        .ok_or_else(|| AppError::Internal("collection not found after update".into()))
}

pub fn sync_schema_columns(
    conn: &Connection,
    collection: &str,
    id: &str,
    schema: &str,
) -> Result<()> {
    if schema.is_empty() || schema == "{}" {
        return Ok(());
    }
    let fields = get_schema_fields(schema);
    if fields.is_empty() {
        return Ok(());
    }
    let set_clause: Vec<String> = fields
        .iter()
        .map(|(field, _)| format!("\"{field}\" = json_extract(data, '$.{field}')"))
        .collect();
    let sql = format!(
        "UPDATE \"_data_{collection}\" SET {} WHERE id = ?1",
        set_clause.join(", ")
    );
    conn.execute(&sql, params![id])
        .map_err(|e| AppError::Internal(format!("sync columns: {e}")))?;
    Ok(())
}

pub fn build_search_clause(search: &str) -> Option<(String, Vec<String>)> {
    let words: Vec<String> = search
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| format!("%{}%", w))
        .collect();
    if words.is_empty() {
        return None;
    }
    let clause = words
        .iter()
        .enumerate()
        .map(|(i, _)| format!("data LIKE ?{}", i + 1))
        .collect::<Vec<_>>()
        .join(" AND ");
    Some((clause, words))
}

fn validate_field(value: &Value, type_str: &str, field_name: &str) -> Option<String> {
    match type_str {
        "string" => {
            if !value.is_string() {
                Some(format!("field '{field_name}' must be a string"))
            } else {
                None
            }
        }
        "number" => {
            if !value.is_number() {
                Some(format!("field '{field_name}' must be a number"))
            } else {
                None
            }
        }
        "integer" => match value.as_f64() {
            Some(f) if f.fract() == 0.0 => None,
            _ => Some(format!("field '{field_name}' must be an integer")),
        },
        "boolean" => {
            if !value.is_boolean() {
                Some(format!("field '{field_name}' must be a boolean"))
            } else {
                None
            }
        }
        "array" => {
            if !value.is_array() {
                Some(format!("field '{field_name}' must be an array"))
            } else {
                None
            }
        }
        "object" => {
            if !value.is_object() {
                Some(format!("field '{field_name}' must be an object"))
            } else {
                None
            }
        }
        "email" => match value.as_str() {
            Some(s) if s.contains('@') => None,
            _ => Some(format!("field '{field_name}' must be a valid email")),
        },
        "url" => match value.as_str() {
            Some(s) if s.starts_with("http://") || s.starts_with("https://") => None,
            _ => Some(format!("field '{field_name}' must be a valid URL")),
        },
        _ => None,
    }
}

pub fn validate_data(data: &Value, schema: &str) -> Result<()> {
    if schema.is_empty() || schema == "{}" {
        return Ok(());
    }
    let schema_map: serde_json::Map<String, Value> = serde_json::from_str(schema)
        .map_err(|e| AppError::Internal(format!("parse schema: {e}")))?;
    let mut errors = Vec::new();
    for (field, type_val) in &schema_map {
        let type_str = type_val.as_str().unwrap_or("string");
        let is_optional = field.ends_with('?');
        let field_name = if is_optional {
            &field[..field.len() - 1]
        } else {
            field.as_str()
        };
        if is_optional && data.get(field_name).is_none_or(|v| v.is_null()) {
            continue;
        }
        match data.get(field_name) {
            None => {
                if !is_optional {
                    errors.push(format!("field '{field_name}' is required"));
                }
            }
            Some(v) if v.is_null() && is_optional => {}
            Some(v) => {
                if let Some(err) = validate_field(v, type_str, field_name) {
                    errors.push(err);
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::BadRequest(errors.join("; ")))
    }
}

pub fn insert_collection(conn: &Connection, name: &str, schema: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO _collections (name, schema) VALUES (?1, ?2)",
        params![name, schema],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("collection already exists".into())
        } else {
            AppError::Internal(format!("insert collection: {e}"))
        }
    })?;
    Ok(())
}

pub fn get_collection(conn: &Connection, name: &str) -> Result<Option<Collection>> {
    let mut stmt = conn
        .prepare("SELECT name, schema, created_at, updated_at FROM _collections WHERE name = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![name], |row| {
            Ok(Collection {
                name: row.get(0)?,
                schema: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(c)) => Ok(Some(c)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_collections(conn: &Connection) -> Result<Vec<Collection>> {
    let mut stmt = conn
        .prepare("SELECT name, schema, created_at, updated_at FROM _collections ORDER BY name")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Collection {
                name: row.get(0)?,
                schema: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut cols = Vec::new();
    for row in rows {
        cols.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(cols)
}

pub fn delete_collection(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM _collections WHERE name = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let sql = format!("DROP TABLE IF EXISTS \"_data_{name}\"");
    conn.execute(&sql, [])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn insert_record(
    conn: &Connection,
    collection: &str,
    id: &str,
    data: &Value,
) -> Result<Record> {
    let now = Utc::now().to_rfc3339();
    let raw = data.to_string();
    let sql = format!(
        "INSERT INTO \"_data_{collection}\" (id, data, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)"
    );
    conn.execute(&sql, params![id, raw, now, now])
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                AppError::Conflict("record with this id already exists".into())
            } else {
                AppError::Internal(format!("insert record: {e}"))
            }
        })?;
    if let Ok(Some(col)) = get_collection(conn, collection)
        && col.schema != "{}"
    {
        let _ = sync_schema_columns(conn, collection, id, &col.schema);
    }
    Ok(Record {
        id: id.to_string(),
        data: data.clone(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_record(conn: &Connection, collection: &str, id: &str) -> Result<Option<Record>> {
    let sql = format!(
        "SELECT id, data, created_at, updated_at FROM \"_data_{collection}\" WHERE id = ?1"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            let id: String = row.get(0)?;
            let data_str: String = row.get(1)?;
            let created_at: String = row.get(2)?;
            let updated_at: String = row.get(3)?;
            let data: Value = serde_json::from_str(&data_str).unwrap_or(serde_json::json!({}));
            Ok(Record {
                id,
                data,
                created_at,
                updated_at,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(r)) => Ok(Some(r)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_records(
    conn: &Connection,
    collection: &str,
    page: i64,
    per_page: i64,
    search: &str,
) -> Result<RecordsPage> {
    let (where_clause, params_list) = if let Some((clause, words)) = build_search_clause(search) {
        (format!(" WHERE {clause}"), words)
    } else {
        (String::new(), Vec::new())
    };

    let count_sql = format!("SELECT COUNT(*) FROM \"_data_{collection}\"{where_clause}");
    let total: i64 = if params_list.is_empty() {
        conn.query_row(&count_sql, [], |row| row.get(0))
            .map_err(|e| AppError::Internal(e.to_string()))?
    } else {
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_list
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();
        conn.query_row(&count_sql, param_refs.as_slice(), |row| row.get(0))
            .map_err(|e| AppError::Internal(e.to_string()))?
    };

    let total_pages = std::cmp::max(1, (total + per_page - 1) / per_page);
    let offset = (page - 1) * per_page;

    let sql = format!(
        "SELECT id, data, created_at, updated_at FROM \"_data_{collection}\"{where_clause} ORDER BY created_at DESC LIMIT ?{limit_idx} OFFSET ?{offset_idx}",
        limit_idx = params_list.len() + 1,
        offset_idx = params_list.len() + 2,
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    for w in &params_list {
        all_params.push(Box::new(w.clone()));
    }
    all_params.push(Box::new(per_page));
    all_params.push(Box::new(offset));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            let id: String = row.get(0)?;
            let data_str: String = row.get(1)?;
            let created_at: String = row.get(2)?;
            let updated_at: String = row.get(3)?;
            let data: Value = serde_json::from_str(&data_str).unwrap_or(serde_json::json!({}));
            Ok(Record {
                id,
                data,
                created_at,
                updated_at,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }

    Ok(RecordsPage {
        records,
        total,
        page,
        per_page,
        total_pages,
    })
}

pub fn update_record(
    conn: &Connection,
    collection: &str,
    id: &str,
    data: &Value,
) -> Result<Record> {
    let now = Utc::now().to_rfc3339();
    let raw = data.to_string();
    let sql = format!("UPDATE \"_data_{collection}\" SET data = ?1, updated_at = ?2 WHERE id = ?3");
    conn.execute(&sql, params![raw, now, id])
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Ok(Some(col)) = get_collection(conn, collection)
        && col.schema != "{}"
    {
        let _ = sync_schema_columns(conn, collection, id, &col.schema);
    }
    let existing = get_record(conn, collection, id)?;
    existing.ok_or_else(|| AppError::NotFound("record not found after update".into()))
}

pub fn delete_record(conn: &Connection, collection: &str, id: &str) -> Result<()> {
    let sql = format!("DELETE FROM \"_data_{collection}\" WHERE id = ?1");
    conn.execute(&sql, params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

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

pub fn insert_bucket(conn: &Connection, name: &str, is_public: bool) -> Result<Bucket> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _buckets (name, is_public, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, is_public as i64, now, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("bucket already exists".into())
        } else {
            AppError::Internal(format!("insert bucket: {e}"))
        }
    })?;
    Ok(Bucket {
        name: name.to_string(),
        is_public,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_bucket(conn: &Connection, name: &str) -> Result<Option<Bucket>> {
    let mut stmt = conn
        .prepare("SELECT name, is_public, created_at, updated_at FROM _buckets WHERE name = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![name], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                is_public: row.get::<_, i64>(1)? != 0,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(b)) => Ok(Some(b)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_buckets(conn: &Connection) -> Result<Vec<Bucket>> {
    let mut stmt = conn
        .prepare("SELECT name, is_public, created_at, updated_at FROM _buckets ORDER BY name")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                is_public: row.get::<_, i64>(1)? != 0,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut buckets = Vec::new();
    for row in rows {
        buckets.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(buckets)
}

pub fn delete_bucket(conn: &Connection, name: &str) -> Result<Vec<FileRecord>> {
    let files = list_files_all(conn, name)?;
    conn.execute("DELETE FROM _files WHERE bucket = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM _buckets WHERE name = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(files)
}

fn list_files_all(conn: &Connection, bucket: &str) -> Result<Vec<FileRecord>> {
    let mut stmt = conn
        .prepare("SELECT id, bucket, filename, mime_type, size, created_at, updated_at FROM _files WHERE bucket = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![bucket], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                bucket: row.get(1)?,
                filename: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut files = Vec::new();
    for row in rows {
        files.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(files)
}

pub fn insert_file(
    conn: &Connection,
    bucket: &str,
    id: &str,
    filename: &str,
    mime_type: &str,
    size: i64,
) -> Result<FileRecord> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _files (id, bucket, filename, mime_type, size, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, bucket, filename, mime_type, size, now, now],
    )
    .map_err(|e| AppError::Internal(format!("insert file: {e}")))?;
    Ok(FileRecord {
        id: id.to_string(),
        bucket: bucket.to_string(),
        filename: filename.to_string(),
        mime_type: mime_type.to_string(),
        size,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_file(conn: &Connection, id: &str) -> Result<Option<FileRecord>> {
    let mut stmt = conn
        .prepare("SELECT id, bucket, filename, mime_type, size, created_at, updated_at FROM _files WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                bucket: row.get(1)?,
                filename: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(f)) => Ok(Some(f)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_files(conn: &Connection, bucket: &str, page: i64, per_page: i64) -> Result<FilesPage> {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _files WHERE bucket = ?1",
            params![bucket],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total_pages = std::cmp::max(1, (total + per_page - 1) / per_page);
    let offset = (page - 1) * per_page;

    let mut stmt = conn
        .prepare(
            "SELECT id, bucket, filename, mime_type, size, created_at, updated_at FROM _files WHERE bucket = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows = stmt
        .query_map(params![bucket, per_page, offset], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                bucket: row.get(1)?,
                filename: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut files = Vec::new();
    for row in rows {
        files.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }

    Ok(FilesPage {
        files,
        total,
        page,
        per_page,
        total_pages,
    })
}

pub fn delete_file(conn: &Connection, id: &str) -> Result<Option<FileRecord>> {
    let existing = get_file(conn, id)?;
    if existing.is_some() {
        conn.execute("DELETE FROM _files WHERE id = ?1", params![id])
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(existing)
}

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

// --- Secrets ---

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

// --- WebSocket ---

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

// --- Cache ---

pub fn set_cache_entry(conn: &Connection, key: &str, value: &str, ttl: i64) -> Result<CacheEntry> {
    let now = chrono::Utc::now().to_rfc3339();
    let expires_at = if ttl > 0 {
        (chrono::Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339()
    } else {
        String::new()
    };
    conn.execute(
        "INSERT INTO _cache (key, value, ttl, expires_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, ttl = excluded.ttl, expires_at = excluded.expires_at, updated_at = excluded.updated_at",
        params![key, value, ttl, expires_at, now, now],
    )
    .map_err(|e| AppError::Internal(format!("set cache: {e}")))?;
    Ok(CacheEntry {
        key: key.to_string(),
        value: value.to_string(),
        ttl,
        expires_at,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_cache_entry(conn: &Connection, key: &str) -> Result<Option<CacheEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT key, value, ttl, expires_at, created_at, updated_at FROM _cache WHERE key = ?1",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![key], |row| {
            Ok(CacheEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                ttl: row.get(2)?,
                expires_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(entry)) => {
            if entry.ttl > 0
                && !entry.expires_at.is_empty()
                && let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&entry.expires_at)
                && chrono::Utc::now() > expires
            {
                conn.execute("DELETE FROM _cache WHERE key = ?1", params![key])
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                return Ok(None);
            }
            Ok(Some(entry))
        }
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn delete_cache_entry(conn: &Connection, key: &str) -> Result<bool> {
    let n = conn
        .execute("DELETE FROM _cache WHERE key = ?1", params![key])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(n > 0)
}

pub fn list_cache_entries(conn: &Connection) -> Result<Vec<CacheEntry>> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut stmt = conn
        .prepare("SELECT key, value, ttl, expires_at, created_at, updated_at FROM _cache WHERE ttl = 0 OR expires_at > ?1 ORDER BY updated_at DESC")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![now], |row| {
            Ok(CacheEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                ttl: row.get(2)?,
                expires_at: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(entries)
}

pub fn flush_cache(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM _cache", [])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

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

// PubSub DB functions

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

// --- Cron Jobs ---

pub fn list_cron_jobs(conn: &Connection) -> Result<Vec<CronJob>> {
    let mut stmt = conn
        .prepare("SELECT id, name, schedule, command, method, headers, body, is_active, last_run_at, last_run_status, created_at, updated_at FROM _cronjobs ORDER BY name")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CronJob {
                id: row.get(0)?,
                name: row.get(1)?,
                schedule: row.get(2)?,
                command: row.get(3)?,
                method: row.get(4)?,
                headers: row.get(5)?,
                body: row.get(6)?,
                is_active: row.get::<_, i64>(7)? != 0,
                last_run_at: row.get(8)?,
                last_run_status: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut jobs = Vec::new();
    for row in rows {
        jobs.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(jobs)
}

pub fn get_cron_job(conn: &Connection, id: &str) -> Result<Option<CronJob>> {
    let mut stmt = conn
        .prepare("SELECT id, name, schedule, command, method, headers, body, is_active, last_run_at, last_run_status, created_at, updated_at FROM _cronjobs WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(CronJob {
                id: row.get(0)?,
                name: row.get(1)?,
                schedule: row.get(2)?,
                command: row.get(3)?,
                method: row.get(4)?,
                headers: row.get(5)?,
                body: row.get(6)?,
                is_active: row.get::<_, i64>(7)? != 0,
                last_run_at: row.get(8)?,
                last_run_status: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(j)) => Ok(Some(j)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn insert_cron_job(
    conn: &Connection,
    id: &str,
    name: &str,
    schedule: &str,
    command: &str,
    method: &str,
    headers: &str,
    body: &str,
) -> Result<CronJob> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _cronjobs (id, name, schedule, command, method, headers, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, name, schedule, command, method, headers, body, now, now],
    )
    .map_err(|e| AppError::Internal(format!("insert cron job: {e}")))?;
    Ok(CronJob {
        id: id.to_string(),
        name: name.to_string(),
        schedule: schedule.to_string(),
        command: command.to_string(),
        method: method.to_string(),
        headers: headers.to_string(),
        body: body.to_string(),
        is_active: true,
        last_run_at: String::new(),
        last_run_status: String::new(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn update_cron_job(
    conn: &Connection,
    id: &str,
    name: &str,
    schedule: &str,
    command: &str,
    method: &str,
    headers: &str,
    body: &str,
    is_active: bool,
) -> Result<CronJob> {
    let now = Utc::now().to_rfc3339();
    let active = if is_active { 1 } else { 0 };
    conn.execute(
        "UPDATE _cronjobs SET name = ?1, schedule = ?2, command = ?3, method = ?4, headers = ?5, body = ?6, is_active = ?7, updated_at = ?8 WHERE id = ?9",
        params![name, schedule, command, method, headers, body, active, now, id],
    )
    .map_err(|e| AppError::Internal(format!("update cron job: {e}")))?;
    get_cron_job(conn, id)?
        .ok_or_else(|| AppError::Internal("cron job not found after update".into()))
}

pub fn delete_cron_job(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM _cronjob_logs WHERE cronjob_id = ?1",
        params![id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM _cronjobs WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn update_cron_job_last_run(
    conn: &Connection,
    id: &str,
    last_run_at: &str,
    status: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE _cronjobs SET last_run_at = ?1, last_run_status = ?2, updated_at = ?3 WHERE id = ?4",
        params![last_run_at, status, now, id],
    )
    .map_err(|e| AppError::Internal(format!("update cron job last run: {e}")))?;
    Ok(())
}

pub fn list_cron_job_logs(conn: &Connection, cronjob_id: &str) -> Result<Vec<CronJobLog>> {
    let mut stmt = conn
        .prepare("SELECT id, cronjob_id, started_at, finished_at, duration_ms, status, output, error FROM _cronjob_logs WHERE cronjob_id = ?1 ORDER BY started_at DESC LIMIT 50")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![cronjob_id], |row| {
            Ok(CronJobLog {
                id: row.get(0)?,
                cronjob_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                duration_ms: row.get(4)?,
                status: row.get(5)?,
                output: row.get(6)?,
                error: row.get(7)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(logs)
}

pub fn insert_cron_job_log(conn: &Connection, log: &CronJobLog) -> Result<()> {
    conn.execute(
        "INSERT INTO _cronjob_logs (id, cronjob_id, started_at, finished_at, duration_ms, status, output, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![log.id, log.cronjob_id, log.started_at, log.finished_at, log.duration_ms, log.status, log.output, log.error],
    )
    .map_err(|e| AppError::Internal(format!("insert cron job log: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_schema_fields_empty() {
        assert!(get_schema_fields("").is_empty());
    }

    #[test]
    fn test_get_schema_fields_empty_object() {
        assert!(get_schema_fields("{}").is_empty());
    }

    #[test]
    fn test_get_schema_fields_single_string() {
        let fields = get_schema_fields(r#"{"name": "string"}"#);
        assert_eq!(fields, vec![("name".to_string(), "TEXT".to_string())]);
    }

    #[test]
    fn test_get_schema_fields_optional_strips_question() {
        let fields = get_schema_fields(r#"{"name?": "string"}"#);
        assert_eq!(fields, vec![("name".to_string(), "TEXT".to_string())]);
    }

    #[test]
    fn test_get_schema_fields_type_mappings() {
        let fields: Vec<(String, String)> =
            get_schema_fields(r#"{"a": "string", "b": "number", "c": "integer", "d": "boolean"}"#);
        assert_eq!(fields.len(), 4);
        let map: std::collections::HashMap<&str, &str> = fields
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(map.get("a"), Some(&"TEXT"));
        assert_eq!(map.get("b"), Some(&"REAL"));
        assert_eq!(map.get("c"), Some(&"INTEGER"));
        assert_eq!(map.get("d"), Some(&"INTEGER"));
    }

    #[test]
    fn test_get_schema_fields_unknown_type_defaults_to_text() {
        let fields = get_schema_fields(r#"{"x": "unknown_type"}"#);
        assert_eq!(fields, vec![("x".to_string(), "TEXT".to_string())]);
    }

    #[test]
    fn test_get_schema_fields_mixed_types() {
        let fields = get_schema_fields(
            r#"{"email": "email", "url": "url", "tags": "array", "meta": "object"}"#,
        );
        let map: std::collections::HashMap<&str, &str> = fields
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(map.len(), 4);
        for v in map.values() {
            assert_eq!(*v, "TEXT");
        }
    }

    #[test]
    fn test_build_search_clause_empty() {
        assert_eq!(build_search_clause(""), None);
    }

    #[test]
    fn test_build_search_clause_whitespace() {
        assert_eq!(build_search_clause("   "), None);
    }

    #[test]
    fn test_build_search_clause_single_word() {
        let (clause, params) = build_search_clause("hello").unwrap();
        assert_eq!(clause, "data LIKE ?1");
        assert_eq!(params, vec!["%hello%"]);
    }

    #[test]
    fn test_build_search_clause_multiple_words() {
        let (clause, params) = build_search_clause("hello world").unwrap();
        assert_eq!(clause, "data LIKE ?1 AND data LIKE ?2");
        assert_eq!(params, vec!["%hello%", "%world%"]);
    }

    #[test]
    fn test_build_search_clause_special_characters() {
        let (clause, params) = build_search_clause("foo%_bar").unwrap();
        assert_eq!(clause, "data LIKE ?1");
        assert_eq!(params, vec!["%foo%_bar%"]);
    }

    #[test]
    fn test_validate_field_string_valid() {
        assert_eq!(validate_field(&json!("hello"), "string", "name"), None);
    }

    #[test]
    fn test_validate_field_string_invalid() {
        let err = validate_field(&json!(42), "string", "name");
        assert_eq!(err, Some("field 'name' must be a string".to_string()));
    }

    #[test]
    fn test_validate_field_number_valid() {
        assert_eq!(validate_field(&json!(4.2), "number", "x"), None);
    }

    #[test]
    fn test_validate_field_number_invalid() {
        let err = validate_field(&json!("text"), "number", "x");
        assert_eq!(err, Some("field 'x' must be a number".to_string()));
    }

    #[test]
    fn test_validate_field_integer_valid() {
        assert_eq!(validate_field(&json!(42), "integer", "n"), None);
    }

    #[test]
    fn test_validate_field_integer_float() {
        let err = validate_field(&json!(3.5), "integer", "n");
        assert_eq!(err, Some("field 'n' must be an integer".to_string()));
    }

    #[test]
    fn test_validate_field_integer_non_number() {
        let err = validate_field(&json!("bad"), "integer", "n");
        assert_eq!(err, Some("field 'n' must be an integer".to_string()));
    }

    #[test]
    fn test_validate_field_boolean_valid() {
        assert_eq!(validate_field(&json!(true), "boolean", "b"), None);
        assert_eq!(validate_field(&json!(false), "boolean", "b"), None);
    }

    #[test]
    fn test_validate_field_boolean_invalid() {
        let err = validate_field(&json!("yes"), "boolean", "b");
        assert_eq!(err, Some("field 'b' must be a boolean".to_string()));
    }

    #[test]
    fn test_validate_field_array_valid() {
        assert_eq!(validate_field(&json!([1, 2, 3]), "array", "a"), None);
    }

    #[test]
    fn test_validate_field_array_invalid() {
        let err = validate_field(&json!("not array"), "array", "a");
        assert_eq!(err, Some("field 'a' must be an array".to_string()));
    }

    #[test]
    fn test_validate_field_object_valid() {
        assert_eq!(validate_field(&json!({"k": "v"}), "object", "o"), None);
    }

    #[test]
    fn test_validate_field_object_invalid() {
        let err = validate_field(&json!("not object"), "object", "o");
        assert_eq!(err, Some("field 'o' must be an object".to_string()));
    }

    #[test]
    fn test_validate_field_email_valid() {
        assert_eq!(
            validate_field(&json!("user@example.com"), "email", "e"),
            None
        );
    }

    #[test]
    fn test_validate_field_email_missing_at() {
        let err = validate_field(&json!("userexample.com"), "email", "e");
        assert_eq!(err, Some("field 'e' must be a valid email".to_string()));
    }

    #[test]
    fn test_validate_field_email_non_string() {
        let err = validate_field(&json!(42), "email", "e");
        assert_eq!(err, Some("field 'e' must be a valid email".to_string()));
    }

    #[test]
    fn test_validate_field_url_valid_http() {
        assert_eq!(
            validate_field(&json!("http://example.com"), "url", "u"),
            None
        );
    }

    #[test]
    fn test_validate_field_url_valid_https() {
        assert_eq!(
            validate_field(&json!("https://example.com"), "url", "u"),
            None
        );
    }

    #[test]
    fn test_validate_field_url_no_protocol() {
        let err = validate_field(&json!("example.com"), "url", "u");
        assert_eq!(err, Some("field 'u' must be a valid URL".to_string()));
    }

    #[test]
    fn test_validate_field_unknown_type() {
        assert_eq!(validate_field(&json!("anything"), "custom_type", "c"), None);
    }

    #[test]
    fn test_validate_data_empty_schema() {
        assert_eq!(validate_data(&json!({"x": 1}), "").ok(), Some(()));
        assert_eq!(validate_data(&json!({"x": 1}), "{}").ok(), Some(()));
    }

    #[test]
    fn test_validate_data_all_valid() {
        let data = json!({"name": "Alice", "age": 30, "active": true});
        let schema = r#"{"name": "string", "age": "integer", "active": "boolean"}"#;
        assert!(validate_data(&data, schema).is_ok());
    }

    #[test]
    fn test_validate_data_missing_required() {
        let data = json!({"name": "Alice"});
        let schema = r#"{"name": "string", "age": "integer"}"#;
        let err = validate_data(&data, schema).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
        assert!(format!("{:?}", err).contains("age"));
    }

    #[test]
    fn test_validate_data_optional_missing() {
        let data = json!({"name": "Alice"});
        let schema = r#"{"name": "string", "age?": "integer"}"#;
        assert!(validate_data(&data, schema).is_ok());
    }

    #[test]
    fn test_validate_data_optional_null() {
        let data = json!({"name": "Alice", "age": null});
        let schema = r#"{"name": "string", "age?": "integer"}"#;
        assert!(validate_data(&data, schema).is_ok());
    }

    #[test]
    fn test_validate_data_type_mismatch() {
        let data = json!({"name": 42});
        let schema = r#"{"name": "string"}"#;
        let err = validate_data(&data, schema).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
        assert!(format!("{:?}", err).contains("must be a string"));
    }

    #[test]
    fn test_validate_data_multiple_errors() {
        let data = json!({"a": 1, "b": "not bool", "c": [1,2,3]});
        let schema = r#"{"a": "string", "b": "boolean", "c": "number"}"#;
        let err = validate_data(&data, schema).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
        let msg = format!("{:?}", err);
        assert!(msg.contains("a") && msg.contains("b") && msg.contains("c"));
    }

    // --- data_dir ---

    #[test]
    fn test_data_dir_default() {
        unsafe {
            std::env::set_var("BLAZERS_DATA", "/tmp/test_blazers");
        }
        let d = data_dir();
        assert_eq!(d, std::path::PathBuf::from("/tmp/test_blazers"));
        unsafe {
            std::env::remove_var("BLAZERS_DATA");
        }
    }

    #[test]
    fn test_data_dir_fallback_home() {
        unsafe {
            std::env::remove_var("BLAZERS_DATA");
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let d = data_dir();
        assert_eq!(d, std::path::PathBuf::from(home).join(".blazers"));
    }

    // --- Collection CRUD ---

    #[test]
    fn test_insert_get_list_delete_collection() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "test_coll", r#"{"name": "string"}"#).unwrap();

        let col = get_collection(&conn, "test_coll").unwrap().unwrap();
        assert_eq!(col.name, "test_coll");
        assert_eq!(col.schema, r#"{"name": "string"}"#);

        let cols = list_collections(&conn).unwrap();
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].name, "test_coll");

        delete_collection(&conn, "test_coll").unwrap();
        assert!(get_collection(&conn, "test_coll").unwrap().is_none());
        assert!(list_collections(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_insert_collection_duplicate() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "dup", "{}").unwrap();
        let err = insert_collection(&conn, "dup", "{}").unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn test_get_collection_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(get_collection(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_list_collections_empty() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        let cols = list_collections(&conn).unwrap();
        assert!(cols.is_empty());
    }

    // --- create_collection_table & migrate_collection_schema & update_collection & sync_schema_columns ---

    #[test]
    fn test_create_collection_table_and_migrate() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "schema_test", r#"{"name": "string"}"#).unwrap();
        create_collection_table(&conn, "schema_test", r#"{"name": "string"}"#).unwrap();

        migrate_collection_schema(
            &conn,
            "schema_test",
            r#"{"name": "string"}"#,
            r#"{"name": "string", "age": "integer"}"#,
        )
        .unwrap();

        let col = get_collection(&conn, "schema_test").unwrap().unwrap();
        assert_eq!(col.schema, r#"{"name": "string", "age": "integer"}"#);
    }

    #[test]
    fn test_update_collection_with_new_schema() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "upd_coll", r#"{"name": "string"}"#).unwrap();
        create_collection_table(&conn, "upd_coll", r#"{"name": "string"}"#).unwrap();

        let updated = update_collection(
            &conn,
            "upd_coll",
            Some(r#"{"name": "string", "active": "boolean"}"#),
        )
        .unwrap();
        assert!(updated.schema.contains("active"));
    }

    #[test]
    fn test_update_collection_no_schema_change() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "nochange", r#"{"x": "string"}"#).unwrap();
        create_collection_table(&conn, "nochange", r#"{"x": "string"}"#).unwrap();

        let updated = update_collection(&conn, "nochange", None).unwrap();
        assert_eq!(updated.schema, r#"{"x": "string"}"#);
    }

    #[test]
    fn test_sync_schema_columns() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "sync_coll", r#"{"val": "string"}"#).unwrap();
        create_collection_table(&conn, "sync_coll", r#"{"val": "string"}"#).unwrap();

        let rec = insert_record(
            &conn,
            "sync_coll",
            "r1",
            &serde_json::json!({"val": "hello"}),
        )
        .unwrap();
        assert_eq!(rec.id, "r1");

        sync_schema_columns(&conn, "sync_coll", "r1", r#"{"val": "string"}"#).unwrap();
        let fetched = get_record(&conn, "sync_coll", "r1").unwrap().unwrap();
        assert_eq!(fetched.data["val"], "hello");
    }

    // --- Record CRUD ---

    #[test]
    fn test_record_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "records", "{}").unwrap();
        create_collection_table(&conn, "records", "{}").unwrap();

        let data = serde_json::json!({"title": "hello", "count": 42});
        let rec = insert_record(&conn, "records", "rec1", &data).unwrap();
        assert_eq!(rec.id, "rec1");
        assert_eq!(rec.data["title"], "hello");

        let fetched = get_record(&conn, "records", "rec1").unwrap().unwrap();
        assert_eq!(fetched.id, "rec1");
        assert_eq!(fetched.data["count"], 42);

        let updated_data = serde_json::json!({"title": "updated", "count": 99});
        let updated = update_record(&conn, "records", "rec1", &updated_data).unwrap();
        assert_eq!(updated.data["title"], "updated");

        let page = list_records(&conn, "records", 1, 10, "").unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.records.len(), 1);

        delete_record(&conn, "records", "rec1").unwrap();
        assert!(get_record(&conn, "records", "rec1").unwrap().is_none());
    }

    #[test]
    fn test_record_search() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "search_test", "{}").unwrap();
        create_collection_table(&conn, "search_test", "{}").unwrap();

        insert_record(
            &conn,
            "search_test",
            "a",
            &serde_json::json!({"text": "hello world"}),
        )
        .unwrap();
        insert_record(
            &conn,
            "search_test",
            "b",
            &serde_json::json!({"text": "goodbye world"}),
        )
        .unwrap();

        let page = list_records(&conn, "search_test", 1, 10, "hello").unwrap();
        assert_eq!(page.total, 1);

        let page2 = list_records(&conn, "search_test", 1, 10, "world").unwrap();
        assert_eq!(page2.total, 2);
    }

    #[test]
    fn test_record_pagination() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "pagination", "{}").unwrap();
        create_collection_table(&conn, "pagination", "{}").unwrap();

        for i in 0..5 {
            insert_record(
                &conn,
                "pagination",
                &format!("r{i}"),
                &serde_json::json!({"n": i}),
            )
            .unwrap();
        }

        let page = list_records(&conn, "pagination", 1, 2, "").unwrap();
        assert_eq!(page.records.len(), 2);
        assert_eq!(page.total, 5);
        assert_eq!(page.total_pages, 3);
    }

    #[test]
    fn test_get_record_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_collection(&conn, "nf_test", "{}").unwrap();
        create_collection_table(&conn, "nf_test", "{}").unwrap();

        assert!(
            get_record(&conn, "nf_test", "nonexistent")
                .unwrap()
                .is_none()
        );
    }

    // --- Bucket CRUD ---

    #[test]
    fn test_bucket_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let bucket = insert_bucket(&conn, "my_bucket", true).unwrap();
        assert_eq!(bucket.name, "my_bucket");
        assert!(bucket.is_public);

        let fetched = get_bucket(&conn, "my_bucket").unwrap().unwrap();
        assert_eq!(fetched.name, "my_bucket");
        assert!(fetched.is_public);

        let buckets = list_buckets(&conn).unwrap();
        assert_eq!(buckets.len(), 1);

        let files = delete_bucket(&conn, "my_bucket").unwrap();
        assert!(files.is_empty());
        assert!(get_bucket(&conn, "my_bucket").unwrap().is_none());
    }

    #[test]
    fn test_bucket_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(get_bucket(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_bucket_duplicate() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_bucket(&conn, "dup_bucket", false).unwrap();
        let err = insert_bucket(&conn, "dup_bucket", false).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    // --- File CRUD ---

    #[test]
    fn test_file_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_bucket(&conn, "files_bucket", false).unwrap();

        let file = insert_file(&conn, "files_bucket", "f1", "test.txt", "text/plain", 100).unwrap();
        assert_eq!(file.id, "f1");
        assert_eq!(file.filename, "test.txt");

        let fetched = get_file(&conn, "f1").unwrap().unwrap();
        assert_eq!(fetched.bucket, "files_bucket");

        let page = list_files(&conn, "files_bucket", 1, 10).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.files.len(), 1);

        let deleted = delete_file(&conn, "f1").unwrap().unwrap();
        assert_eq!(deleted.id, "f1");
        assert!(get_file(&conn, "f1").unwrap().is_none());
    }

    #[test]
    fn test_delete_file_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(delete_file(&conn, "nonexistent").unwrap().is_none());
    }

    // --- Cache CRUD ---

    #[test]
    fn test_cache_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let entry = set_cache_entry(&conn, "k1", "v1", 0).unwrap();
        assert_eq!(entry.key, "k1");
        assert_eq!(entry.ttl, 0);

        let fetched = get_cache_entry(&conn, "k1").unwrap().unwrap();
        assert_eq!(fetched.value, "v1");

        let deleted = delete_cache_entry(&conn, "k1").unwrap();
        assert!(deleted);
        assert!(get_cache_entry(&conn, "k1").unwrap().is_none());

        let deleted2 = delete_cache_entry(&conn, "nonexistent").unwrap();
        assert!(!deleted2);
    }

    #[test]
    fn test_cache_list_and_flush() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        set_cache_entry(&conn, "a", "1", 0).unwrap();
        set_cache_entry(&conn, "b", "2", 0).unwrap();

        let entries = list_cache_entries(&conn).unwrap();
        assert_eq!(entries.len(), 2);

        flush_cache(&conn).unwrap();
        assert!(list_cache_entries(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_cache_upsert() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        set_cache_entry(&conn, "k", "old", 0).unwrap();
        set_cache_entry(&conn, "k", "new", 3600).unwrap();

        let entry = get_cache_entry(&conn, "k").unwrap().unwrap();
        assert_eq!(entry.value, "new");
        assert_eq!(entry.ttl, 3600);
    }

    // --- Notification CRUD ---

    #[test]
    fn test_notification_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let n = insert_notification(&conn, "n1", "Hello", "Body text", "info").unwrap();
        assert_eq!(n.title, "Hello");
        assert!(!n.is_read);

        let fetched = get_notification(&conn, "n1").unwrap().unwrap();
        assert_eq!(fetched.body, "Body text");

        let list = list_notifications(&conn).unwrap();
        assert_eq!(list.len(), 1);

        let read = update_notification_read(&conn, "n1").unwrap();
        assert!(read.is_read);

        let deleted = delete_notification(&conn, "n1").unwrap();
        assert!(deleted);
        assert!(get_notification(&conn, "n1").unwrap().is_none());

        let deleted_false = delete_notification(&conn, "nonexistent").unwrap();
        assert!(!deleted_false);
    }

    #[test]
    fn test_notification_clear_all() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_notification(&conn, "n1", "A", "", "info").unwrap();
        insert_notification(&conn, "n2", "B", "", "warn").unwrap();

        clear_notifications(&conn).unwrap();
        let list = list_notifications(&conn).unwrap();
        assert!(list.is_empty());
    }

    // --- Webhook CRUD ---

    #[test]
    fn test_webhook_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let wh = insert_webhook(
            &conn,
            "wh1",
            "test hook",
            "http://example.com",
            &["event.a".to_string()],
            "sec123",
        )
        .unwrap();
        assert_eq!(wh.name, "test hook");
        assert!(wh.is_active);

        let hooks = list_webhooks(&conn).unwrap();
        assert_eq!(hooks.len(), 1);

        let fetched = get_webhook(&conn, "wh1").unwrap().unwrap();
        assert_eq!(fetched.url, "http://example.com");

        let updated = update_webhook(
            &conn,
            "wh1",
            "updated hook",
            "http://example.net",
            &["event.b".to_string()],
            "newsec",
            false,
        )
        .unwrap();
        assert!(!updated.is_active);
        assert_eq!(updated.name, "updated hook");

        delete_webhook(&conn, "wh1").unwrap();
        assert!(get_webhook(&conn, "wh1").unwrap().is_none());
        assert!(list_webhooks(&conn).unwrap().is_empty());
    }

    // --- Webhook Log ---

    #[test]
    fn test_webhook_log_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let log = WebhookLog {
            id: "log1".into(),
            webhook_id: "wh1".into(),
            event: "event.a".into(),
            url: "http://example.com".into(),
            request_body: "{}".into(),
            response_status: 200,
            response_body: "ok".into(),
            error: String::new(),
            status: "delivered".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        insert_webhook_log(&conn, &log).unwrap();

        let logs = list_webhook_logs(&conn, "wh1").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].event, "event.a");
    }

    // --- WS Connection CRUD ---

    #[test]
    fn test_ws_connection_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_ws_connection(&conn, "ws1", "127.0.0.1", "/chat", "test-agent").unwrap();

        let conns = list_ws_connections(&conn).unwrap();
        assert_eq!(conns.len(), 1);
        assert!(conns[0].is_active);

        let fetched = get_ws_connection(&conn, "ws1").unwrap().unwrap();
        assert_eq!(fetched.remote_addr, "127.0.0.1");

        update_ws_disconnect(&conn, "ws1").unwrap();
        let after = get_ws_connection(&conn, "ws1").unwrap().unwrap();
        assert!(!after.is_active);
        assert!(!after.disconnected_at.is_empty());

        delete_ws_connection(&conn, "ws1").unwrap();
        assert!(get_ws_connection(&conn, "ws1").unwrap().is_none());
    }

    #[test]
    fn test_get_ws_connection_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(get_ws_connection(&conn, "nonexistent").unwrap().is_none());
    }

    // --- WS Messages ---

    #[test]
    fn test_ws_message_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_ws_connection(&conn, "ws_msg", "10.0.0.1", "/", "agent").unwrap();
        insert_ws_message(&conn, "ws_msg", "sent", "hello").unwrap();
        insert_ws_message(&conn, "ws_msg", "received", "world").unwrap();

        let msgs = list_ws_messages(&conn, "ws_msg").unwrap();
        assert_eq!(msgs.len(), 2);

        let all = list_all_ws_messages(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    // --- PubSub Topic CRUD ---

    #[test]
    fn test_pubsub_topic_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let topic = insert_pubsub_topic(&conn, "t1", "my_topic").unwrap();
        assert_eq!(topic.name, "my_topic");

        let topics = list_pubsub_topics(&conn).unwrap();
        assert_eq!(topics.len(), 1);

        let by_name = get_pubsub_topic_by_name(&conn, "my_topic")
            .unwrap()
            .unwrap();
        assert_eq!(by_name.id, "t1");

        delete_pubsub_topic(&conn, "my_topic").unwrap();
        assert!(list_pubsub_topics(&conn).unwrap().is_empty());
    }

    // --- PubSub Messages ---

    #[test]
    fn test_pubsub_message_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_pubsub_topic(&conn, "topic1", "chat").unwrap();
        insert_pubsub_message(&conn, "m1", "topic1", "hello world").unwrap();

        let msgs = list_pubsub_messages(&conn, "topic1").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].body, "hello world");
    }

    // --- Cron Job CRUD ---

    #[test]
    fn test_cron_job_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let job = insert_cron_job(
            &conn,
            "cj1",
            "daily_task",
            "0 0 * * *",
            "echo hi",
            "GET",
            "",
            "",
        )
        .unwrap();
        assert_eq!(job.name, "daily_task");
        assert!(job.is_active);

        let jobs = list_cron_jobs(&conn).unwrap();
        assert_eq!(jobs.len(), 1);

        let fetched = get_cron_job(&conn, "cj1").unwrap().unwrap();
        assert_eq!(fetched.schedule, "0 0 * * *");

        let updated = update_cron_job(
            &conn,
            "cj1",
            "nightly",
            "0 0 * * 0",
            "echo bye",
            "POST",
            "{}",
            "{}",
            false,
        )
        .unwrap();
        assert!(!updated.is_active);
        assert_eq!(updated.name, "nightly");

        delete_cron_job(&conn, "cj1").unwrap();
        assert!(get_cron_job(&conn, "cj1").unwrap().is_none());
    }

    #[test]
    fn test_cron_job_last_run() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_cron_job(&conn, "cj2", "test_job", "* * * * *", "cmd", "GET", "", "").unwrap();
        update_cron_job_last_run(&conn, "cj2", "2024-01-01T00:00:00Z", "success").unwrap();

        let job = get_cron_job(&conn, "cj2").unwrap().unwrap();
        assert_eq!(job.last_run_at, "2024-01-01T00:00:00Z");
        assert_eq!(job.last_run_status, "success");
    }

    #[test]
    fn test_cron_job_log() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let log = CronJobLog {
            id: "cjl1".into(),
            cronjob_id: "cj1".into(),
            started_at: "2024-01-01T00:00:00Z".into(),
            finished_at: "2024-01-01T01:00:00Z".into(),
            duration_ms: 3600000,
            status: "success".into(),
            output: "done".into(),
            error: String::new(),
        };
        insert_cron_job_log(&conn, &log).unwrap();

        let logs = list_cron_job_logs(&conn, "cj1").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
    }

    // --- Secret CRUD ---

    #[test]
    fn test_secret_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let secret = insert_secret(&conn, "s1", "api_key", "abc123", "general").unwrap();
        assert_eq!(secret.name, "api_key");

        let secrets = list_secrets(&conn).unwrap();
        assert_eq!(secrets.len(), 1);

        let fetched = get_secret(&conn, "s1").unwrap().unwrap();
        assert_eq!(fetched.value, "abc123");

        let updated = update_secret(&conn, "s1", "new_key", "xyz789", "admin").unwrap();
        assert_eq!(updated.name, "new_key");

        let confirm = get_secret(&conn, "s1").unwrap().unwrap();
        assert_eq!(confirm.value, "xyz789");

        delete_secret(&conn, "s1").unwrap();
        assert!(get_secret(&conn, "s1").unwrap().is_none());
    }

    #[test]
    fn test_get_secret_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(get_secret(&conn, "nonexistent").unwrap().is_none());
    }

    // --- User operations ---

    #[test]
    fn test_user_insert_and_find() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_user(&conn, "u1", "user@example.com", "hashed_pw").unwrap();

        let found = find_user_by_email(&conn, "user@example.com")
            .unwrap()
            .unwrap();
        assert_eq!(found.0, "u1");
        assert_eq!(found.1, "user@example.com");
        assert_eq!(found.4, "hashed_pw");
    }

    #[test]
    fn test_find_user_by_email_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(
            find_user_by_email(&conn, "noone@example.com")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_insert_user_duplicate_email() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        insert_user(&conn, "u1", "dup@example.com", "pw").unwrap();
        let err = insert_user(&conn, "u2", "dup@example.com", "pw2").unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    // --- Log operations ---

    #[test]
    fn test_log_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();

        let log = insert_log(&conn, "log1", "info", "server started", "{}").unwrap();
        assert_eq!(log.level, "info");
        assert_eq!(log.message, "server started");

        let logs = list_logs(&conn).unwrap();
        assert_eq!(logs.len(), 1);

        clear_logs(&conn).unwrap();
        assert!(list_logs(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_get_cache_entry_expired() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        set_cache_entry(&conn, "expkey", "oldval", 1).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let result = get_cache_entry(&conn, "expkey").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_collection_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        let err = update_collection(&conn, "nonexistent", Some("{}")).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_update_record_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        insert_collection(&conn, "rc", "{}").unwrap();
        create_collection_table(&conn, "rc", "{}").unwrap();
        let err = update_record(&conn, "rc", "noexist", &json!({"x": 1})).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_update_cron_job_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        let err = update_cron_job(
            &conn,
            "nojob",
            "n",
            "* * * * *",
            "cmd",
            "GET",
            "",
            "",
            false,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Internal(_)));
    }

    #[test]
    fn test_delete_bucket_with_files() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        insert_bucket(&conn, "bf", false).unwrap();
        insert_file(&conn, "bf", "f1", "a.txt", "text/plain", 10).unwrap();
        let files = delete_bucket(&conn, "bf").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].id, "f1");
    }

    #[test]
    fn test_get_webhook_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(get_webhook(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_get_cron_job_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(get_cron_job(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_insert_record_duplicate() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        insert_collection(&conn, "dup_rec", "{}").unwrap();
        create_collection_table(&conn, "dup_rec", "{}").unwrap();
        insert_record(&conn, "dup_rec", "r1", &json!({"x": 1})).unwrap();
        let err = insert_record(&conn, "dup_rec", "r1", &json!({"x": 2})).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn test_list_webhooks_empty() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(list_webhooks(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_list_webhook_logs_empty() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(list_webhook_logs(&conn, "nonexistent").unwrap().is_empty());
    }

    #[test]
    fn test_list_files_empty() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        let page = list_files(&conn, "nonexistent", 1, 10).unwrap();
        assert!(page.files.is_empty());
        assert_eq!(page.total, 0);
    }

    #[test]
    fn test_list_cron_job_logs_empty() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(list_cron_job_logs(&conn, "nonexistent").unwrap().is_empty());
    }

    #[test]
    fn test_list_pubsub_messages_empty() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(
            list_pubsub_messages(&conn, "nonexistent")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_list_ws_messages_empty() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(list_ws_messages(&conn, "nonexistent").unwrap().is_empty());
    }

    #[test]
    fn test_get_pubsub_topic_by_name_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(
            get_pubsub_topic_by_name(&conn, "nonexistent")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_get_notification_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_db(&conn).unwrap();
        assert!(get_notification(&conn, "nonexistent").unwrap().is_none());
    }
}
