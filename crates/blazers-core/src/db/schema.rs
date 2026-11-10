use chrono::Utc;
use rusqlite::{Connection, params};
use serde_json::Value;
use std::collections::HashMap;

use super::*;
use crate::models::*;

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
