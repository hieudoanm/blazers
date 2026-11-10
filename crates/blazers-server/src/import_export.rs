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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn export_data_empty_round_trip() {
        let data = ExportData {
            collections: vec![],
            records: HashMap::new(),
            buckets: vec![],
            files: vec![],
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ExportData = serde_json::from_str(&json).unwrap();
        assert!(deserialized.collections.is_empty());
        assert!(deserialized.records.is_empty());
        assert!(deserialized.buckets.is_empty());
        assert!(deserialized.files.is_empty());
    }

    #[test]
    fn export_data_with_data_round_trip() {
        let data = ExportData {
            collections: vec![Collection {
                name: "users".into(),
                schema: "{\"name\":\"string\"}".into(),
                created_at: "2024-01-01T00:00:00Z".into(),
                updated_at: "2024-01-01T00:00:00Z".into(),
            }],
            records: {
                let mut map = HashMap::new();
                map.insert(
                    "users".into(),
                    vec![Record {
                        id: "rec_1".into(),
                        data: json!({"name": "Alice"}),
                        created_at: "2024-01-01T00:00:00Z".into(),
                        updated_at: "2024-01-01T00:00:00Z".into(),
                    }],
                );
                map
            },
            buckets: vec![Bucket {
                name: "files".into(),
                is_public: true,
                created_at: "2024-01-01T00:00:00Z".into(),
                updated_at: "2024-01-01T00:00:00Z".into(),
            }],
            files: vec![FileRecord {
                id: "f1".into(),
                bucket: "files".into(),
                filename: "test.txt".into(),
                mime_type: "text/plain".into(),
                size: 100,
                created_at: "2024-01-01T00:00:00Z".into(),
                updated_at: "2024-01-01T00:00:00Z".into(),
            }],
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ExportData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.collections.len(), 1);
        assert_eq!(deserialized.collections[0].name, "users");
        assert_eq!(deserialized.records.len(), 1);
        assert_eq!(deserialized.records["users"].len(), 1);
        assert_eq!(deserialized.records["users"][0].data["name"], "Alice");
        assert_eq!(deserialized.buckets.len(), 1);
        assert_eq!(deserialized.buckets[0].name, "files");
        assert!(deserialized.buckets[0].is_public);
        assert_eq!(deserialized.files.len(), 1);
        assert_eq!(deserialized.files[0].filename, "test.txt");
        assert_eq!(deserialized.files[0].size, 100);
    }

    #[test]
    fn export_data_json_structure() {
        let data = ExportData {
            collections: vec![],
            records: HashMap::new(),
            buckets: vec![],
            files: vec![],
        };
        let value = serde_json::to_value(&data).unwrap();
        assert!(value.is_object());
        assert!(value.get("collections").unwrap().is_array());
        assert!(value.get("records").unwrap().is_object());
        assert!(value.get("buckets").unwrap().is_array());
        assert!(value.get("files").unwrap().is_array());
        assert_eq!(value["collections"].as_array().unwrap().len(), 0);
        assert_eq!(value["records"].as_object().unwrap().len(), 0);
        assert_eq!(value["buckets"].as_array().unwrap().len(), 0);
        assert_eq!(value["files"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn import_query_default_skip_existing() {
        let query: ImportQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.skip_existing, None);
    }

    #[test]
    fn import_query_skip_existing_true() {
        let query: ImportQuery = serde_json::from_str(r#"{"skip_existing": true}"#).unwrap();
        assert_eq!(query.skip_existing, Some(true));
    }

    #[test]
    fn import_query_skip_existing_false() {
        let query: ImportQuery = serde_json::from_str(r#"{"skip_existing": false}"#).unwrap();
        assert_eq!(query.skip_existing, Some(false));
    }
}
