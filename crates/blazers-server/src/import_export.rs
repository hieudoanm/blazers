use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::handlers::AppState;
use blazers_core::db;
use blazers_core::models::*;

/// The wire shape of an export payload, and the payload an import accepts.
///
/// Every field defaults so a caller can import a hand-written partial payload,
/// and so an export written by one port can be read by another that carries
/// fewer sections. Without the defaults a payload missing a single key fails to
/// parse outright instead of importing the sections it does carry.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExportData {
    #[serde(default)]
    pub collections: Vec<Collection>,
    #[serde(default)]
    pub records: HashMap<String, Vec<Record>>,
    #[serde(default)]
    pub buckets: Vec<Bucket>,
    #[serde(default)]
    pub files: Vec<FileRecord>,
}

#[derive(Debug, Deserialize)]
pub struct ImportQuery {
    pub skip_existing: Option<bool>,
}

pub async fn handle_export(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<ExportData>, AppError> {
    crate::handlers::extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let collections = db::list_collections(&conn)?;
    let mut records = HashMap::new();
    for col in &collections {
        let page = db::list_records(&conn, &col.name, 1, 100000, "")?;
        records.insert(col.name.clone(), page.records);
    }
    let buckets = db::list_buckets(&conn)?;
    let mut files = Vec::new();
    for bucket in &buckets {
        let page = db::list_files(&conn, &bucket.name, 1, 100000)?;
        files.extend(page.files);
    }
    Ok(Json(ExportData {
        collections,
        records,
        buckets,
        files,
    }))
}

pub async fn handle_import(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Query(query): Query<ImportQuery>,
    Json(data): Json<ExportData>,
) -> std::result::Result<Json<Value>, AppError> {
    crate::handlers::extract_claims(&headers)?;
    let skip_existing = query.skip_existing.unwrap_or(false);
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut created_collections = 0i64;
    let mut created_records = 0i64;
    let mut created_buckets = 0i64;
    let mut created_files = 0i64;

    for col in &data.collections {
        let existing = db::get_collection(&conn, &col.name)?;
        if existing.is_some() {
            if skip_existing {
                continue;
            }
            return Err(AppError::Conflict(format!(
                "collection '{}' already exists",
                col.name
            )));
        }
        db::insert_collection(&conn, &col.name, &col.schema)?;
        db::create_collection_table(&conn, &col.name, &col.schema)?;
        created_collections += 1;
    }

    for (col_name, recs) in &data.records {
        for rec in recs {
            let existing = db::get_record(&conn, col_name, &rec.id)?;
            if existing.is_some() {
                if skip_existing {
                    continue;
                }
                continue;
            }
            db::insert_record(&conn, col_name, &rec.id, &rec.data)?;
            created_records += 1;
        }
    }

    for bucket in &data.buckets {
        let existing = db::get_bucket(&conn, &bucket.name)?;
        if existing.is_some() {
            if skip_existing {
                continue;
            }
            return Err(AppError::Conflict(format!(
                "bucket '{}' already exists",
                bucket.name
            )));
        }
        db::insert_bucket(&conn, &bucket.name, bucket.is_public)?;
        created_buckets += 1;
    }

    for f in &data.files {
        let existing = db::get_file(&conn, &f.id)?;
        if existing.is_some() {
            if skip_existing {
                continue;
            }
            continue;
        }
        db::insert_file(&conn, &f.bucket, &f.id, &f.filename, &f.mime_type, f.size)?;
        created_files += 1;
    }

    Ok(Json(serde_json::json!({
        "collections": created_collections,
        "records": created_records,
        "buckets": created_buckets,
        "files": created_files,
    })))
}

#[cfg(test)]
mod tests;
