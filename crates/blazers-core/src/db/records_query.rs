//! Filtered/sorted record listing — see `records.rs` for plain CRUD.

use rusqlite::Connection;
use serde_json::Value;

use super::records::list_records;
use super::schema::build_search_clause;
use crate::models::*;

/// List records with optional PB-style `filter` expression and `sort` spec.
/// Filter/sort are evaluated in Rust over the JSON `data` column; `auth` is
/// the `@request.auth` context. Falls back to the SQL path when neither is
/// given so pagination stays cheap for the common case.
#[allow(clippy::too_many_arguments)]
pub fn list_records_query(
    conn: &Connection,
    collection: &str,
    page: i64,
    per_page: i64,
    search: &str,
    filter: Option<&str>,
    sort: Option<&str>,
    auth: &serde_json::Value,
) -> Result<RecordsPage> {
    let has_filter = filter.is_some_and(|f| !f.trim().is_empty());
    let has_sort = sort.is_some_and(|s| !s.trim().is_empty());
    if !has_filter && !has_sort {
        return list_records(conn, collection, page, per_page, search);
    }

    // Fetch every row (optionally pre-filtered by the cheap SQL `search`
    // clause), then apply the expression filter, sort and paginate in memory.
    let (where_clause, params_list) = if let Some((clause, words)) = build_search_clause(search) {
        (format!(" WHERE {clause}"), words)
    } else {
        (String::new(), Vec::new())
    };
    let sql = format!(
        "SELECT id, data, created_at, updated_at FROM \"_data_{collection}\"{where_clause}"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_list
        .iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();
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

    let compiled = crate::filter::compile(filter.unwrap_or(""))
        .map_err(|e| AppError::BadRequest(format!("invalid filter: {e}")))?;

    let mut matched: Vec<Record> = Vec::new();
    for row in rows {
        let rec = row.map_err(|e| AppError::Internal(e.to_string()))?;
        let view = record_view(&rec);
        if compiled.matches(&view, auth) {
            matched.push(rec);
        }
    }

    if let Some(spec) = sort.filter(|s| !s.trim().is_empty()) {
        let keys: Vec<(String, bool)> = spec
            .split(',')
            .map(|part| {
                let p = part.trim();
                if let Some(name) = p.strip_prefix('-') {
                    (name.to_string(), true)
                } else {
                    (p.to_string(), false)
                }
            })
            .collect();
        matched.sort_by(|a, b| {
            for (field, desc) in &keys {
                let va = resolve_path(&record_view(a), field);
                let vb = resolve_path(&record_view(b), field);
                let (ra, rb) = (crate::filter::sort_rank(&va), crate::filter::sort_rank(&vb));
                let ord = ra.cmp(&rb);
                let ord = if *desc { ord.reverse() } else { ord };
                // Null always sorts last regardless of direction.
                let ord = match (va.is_null(), vb.is_null()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => ord,
                };
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        });
    }

    let total = matched.len() as i64;
    let total_pages = std::cmp::max(1, (total + per_page - 1) / per_page);
    let offset = ((page - 1) * per_page) as usize;
    let end = (offset + per_page as usize).min(matched.len());
    let records = if offset >= matched.len() {
        Vec::new()
    } else {
        matched[offset..end].to_vec()
    };

    Ok(RecordsPage {
        records,
        total,
        page,
        per_page,
        total_pages,
    })
}

/// The JSON view filter/sort expressions see: record meta fields plus all
/// `data` keys merged at the top level (data wins on collision except id).
pub fn record_view(rec: &Record) -> Value {
    let mut view = serde_json::Map::new();
    view.insert("id".into(), Value::String(rec.id.clone()));
    view.insert("created".into(), Value::String(rec.created_at.clone()));
    view.insert("updated".into(), Value::String(rec.updated_at.clone()));
    if let Value::Object(map) = &rec.data {
        for (k, v) in map {
            view.insert(k.clone(), v.clone());
        }
    }
    Value::Object(view)
}

fn resolve_path(view: &Value, path: &str) -> Value {
    let mut cur = view.clone();
    for part in path.split('.') {
        cur = match cur.get(part) {
            Some(v) => v.clone(),
            None => return Value::Null,
        };
    }
    cur
}
