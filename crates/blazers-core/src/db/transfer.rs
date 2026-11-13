use std::collections::HashMap;

use rusqlite::Connection;

use super::*;
use crate::models::*;

const EXPORT_PAGE_SIZE: i64 = 1_000_000;

/// Reads every collection, record, bucket and file row into an `ExportData`
/// payload — the same shape the HTTP `/export` route returns.
pub fn export_all(conn: &Connection) -> Result<ExportData> {
    let collections = list_collections(conn)?;
    let mut records = HashMap::new();
    for col in &collections {
        let page = list_records(conn, &col.name, 1, EXPORT_PAGE_SIZE, "")?;
        records.insert(col.name.clone(), page.records);
    }
    let buckets = list_buckets(conn)?;
    let mut files = Vec::new();
    for bucket in &buckets {
        let page = list_files(conn, &bucket.name, 1, EXPORT_PAGE_SIZE)?;
        files.extend(page.files);
    }
    Ok(ExportData {
        collections,
        records,
        buckets,
        files,
    })
}

/// Applies an `ExportData` payload, the inverse of `export_all`.
///
/// With `skip_existing` set, collections and buckets that already exist are
/// skipped; without it they produce `AppError::Conflict`. Records and files
/// that already exist are always skipped.
pub fn import_all(
    conn: &Connection,
    data: &ExportData,
    skip_existing: bool,
) -> Result<TransferSummary> {
    let mut summary = TransferSummary::default();

    for col in &data.collections {
        if get_collection(conn, &col.name)?.is_some() {
            if skip_existing {
                continue;
            }
            return Err(AppError::Conflict(format!(
                "collection '{}' already exists",
                col.name
            )));
        }
        insert_collection(conn, &col.name, &col.schema)?;
        create_collection_table(conn, &col.name, &col.schema)?;
        summary.collections += 1;
    }

    for (col_name, recs) in &data.records {
        for rec in recs {
            if get_record(conn, col_name, &rec.id)?.is_some() {
                continue;
            }
            insert_record(conn, col_name, &rec.id, &rec.data)?;
            summary.records += 1;
        }
    }

    for bucket in &data.buckets {
        if get_bucket(conn, &bucket.name)?.is_some() {
            if skip_existing {
                continue;
            }
            return Err(AppError::Conflict(format!(
                "bucket '{}' already exists",
                bucket.name
            )));
        }
        insert_bucket(conn, &bucket.name, bucket.is_public)?;
        summary.buckets += 1;
    }

    for f in &data.files {
        if get_file(conn, &f.id)?.is_some() {
            continue;
        }
        insert_file(conn, &f.bucket, &f.id, &f.filename, &f.mime_type, f.size)?;
        summary.files += 1;
    }

    Ok(summary)
}
