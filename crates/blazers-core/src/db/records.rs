use chrono::Utc;
use rusqlite::{Connection, params};
use serde_json::Value;

use super::*;
use crate::models::*;

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
