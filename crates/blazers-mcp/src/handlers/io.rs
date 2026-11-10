use super::*;

/// `blazers_export` returns every collection, record, bucket and file, using the
/// same shape the HTTP `/export` route returns.
pub fn handle_export(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: ExportArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    if args.format != "json" {
        return error_result(format!(
            "blazers_export supports only the json format in this build, not '{}'",
            args.format
        ));
    }
    let exported = with_db(db, |conn| -> Result<ExportData, AppError> {
        let collections = db::list_collections(conn)?;
        let mut records = std::collections::HashMap::new();
        for collection in &collections {
            let page = db::list_records(conn, &collection.name, 1, 1_000_000, "")?;
            records.insert(collection.name.clone(), page.records);
        }
        let buckets = db::list_buckets(conn)?;
        let mut files = Vec::new();
        for bucket in &buckets {
            let page = db::list_files(conn, &bucket.name, 1, 1_000_000)?;
            files.extend(page.files);
        }
        Ok(ExportData {
            collections,
            records,
            buckets,
            files,
        })
    });
    match exported {
        Ok(data) => json_result(&serde_json::to_value(data).unwrap_or(Value::Null)),
        Err(message) => error_result(format!("export: {message}")),
    }
}

/// `blazers_import` applies an export payload, the inverse of
/// `blazers_export`. Existing records are overwritten unless `skip_existing` is
/// set, matching the HTTP `/import` route.
pub fn handle_import(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: ImportArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    if args.format != "json" {
        return error_result(format!(
            "blazers_import supports only the json format in this build, not '{}'",
            args.format
        ));
    }
    let payload: ExportData = match serde_json::from_str(&args.data) {
        Ok(payload) => payload,
        Err(err) => return error_result(format!("parse data: {err}")),
    };

    let summary = with_db(db, |conn| -> Result<Value, AppError> {
        let mut created_collections = 0i64;
        let mut created_records = 0i64;
        let mut created_buckets = 0i64;
        let mut created_files = 0i64;
        for collection in &payload.collections {
            if db::get_collection(conn, &collection.name)?.is_some() {
                continue;
            }
            db::insert_collection(conn, &collection.name, &collection.schema)?;
            db::create_collection_table(conn, &collection.name, &collection.schema)?;
            created_collections += 1;
        }
        for (name, records) in &payload.records {
            for record in records {
                let existing = db::get_record(conn, name, &record.id)?;
                match existing {
                    Some(_) => continue,
                    None => {
                        db::insert_record(conn, name, &record.id, &record.data)?;
                        created_records += 1;
                    }
                }
            }
        }
        for bucket in &payload.buckets {
            if db::get_bucket(conn, &bucket.name)?.is_some() {
                continue;
            }
            db::insert_bucket(conn, &bucket.name, bucket.is_public)?;
            created_buckets += 1;
        }
        for file in &payload.files {
            if db::get_file(conn, &file.id)?.is_some() {
                continue;
            }
            db::insert_file(
                conn,
                &file.bucket,
                &file.id,
                &file.filename,
                &file.mime_type,
                file.size,
            )?;
            created_files += 1;
        }
        Ok(json!({
            "created_collections": created_collections,
            "created_records": created_records,
            "created_buckets": created_buckets,
            "created_files": created_files,
        }))
    });
    match summary {
        Ok(value) => json_result(&value),
        Err(message) => error_result(format!("import: {message}")),
    }
}
