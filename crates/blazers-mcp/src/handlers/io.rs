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
        db::export_all(conn)
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
        let s = db::import_all(conn, &payload, true)?;
        Ok(json!({
            "created_collections": s.collections,
            "created_records": s.records,
            "created_buckets": s.buckets,
            "created_files": s.files,
        }))
    });
    match summary {
        Ok(value) => json_result(&value),
        Err(message) => error_result(format!("import: {message}")),
    }
}
