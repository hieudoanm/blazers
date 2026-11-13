use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::Deserialize;
use serde_json::Value;

use crate::handlers::AppState;
use blazers_core::db;
use blazers_core::models::*;

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

    Ok(Json(db::export_all(&conn)?))
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

    let summary = db::import_all(&conn, &data, skip_existing)?;
    Ok(Json(serde_json::json!({
        "collections": summary.collections,
        "records": summary.records,
        "buckets": summary.buckets,
        "files": summary.files,
    })))
}

#[cfg(test)]
mod tests;
