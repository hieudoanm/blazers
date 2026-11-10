use super::*;

/// `blazers_health` reports the server is running and can reach its database.
/// `blazers_health` reports the server is running and can reach its database.
pub fn handle_health(db: &DbHandle, arguments: &str) -> ToolResult {
    let _ = arguments;
    let reachable = with_db(db, |conn| {
        conn.query_row("SELECT 1", [], |row| row.get::<_, i64>(0))
            .map(|_| ())
            .map_err(|err| AppError::Internal(format!("ping failed: {err}")))
    });
    match reachable {
        Ok(()) => text_result("blazers-mcp is operational and the database is reachable"),
        Err(message) => error_result(format!("the database is not reachable: {message}")),
    }
}

/// `blazers_collections_list` returns every collection and its schema.
pub fn handle_collections_list(db: &DbHandle, arguments: &str) -> ToolResult {
    let _ = arguments;
    match with_db(db, db::list_collections) {
        Ok(collections) => json_result(&json!({ "collections": collections })),
        Err(message) => error_result(format!("list collections: {message}")),
    }
}

/// `blazers_collections_create` creates a collection and its backing table.
pub fn handle_collections_create(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: CreateCollectionArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return error_result("collection name must not be empty");
    }

    let outcome = with_db(db, |conn| {
        if db::get_collection(conn, &name)?.is_some() {
            return Err(AppError::Conflict(format!(
                "collection '{name}' already exists"
            )));
        }
        db::insert_collection(conn, &name, &args.schema)?;
        db::create_collection_table(conn, &name, &args.schema)?;
        Ok(())
    });
    match outcome {
        Ok(()) => json_result(&json!({ "name": name, "schema": args.schema, "created": true })),
        Err(message) => error_result(message),
    }
}

/// `blazers_collections_delete` drops a collection and its data table.
pub fn handle_collections_delete(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: NameArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    match with_db(db, |conn| {
        if db::get_collection(conn, &args.name)?.is_none() {
            return Err(AppError::NotFound(format!(
                "collection '{}' not found",
                args.name
            )));
        }
        db::delete_collection(conn, &args.name)?;
        Ok(())
    }) {
        Ok(()) => json_result(&json!({ "name": args.name, "deleted": true })),
        Err(message) => error_result(message),
    }
}

/// `blazers_records_list` returns one page of records.
pub fn handle_records_list(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: ListRecordsArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    if args.page < 1 {
        return error_result("page must be 1 or greater");
    }
    if !(1..=MAX_PER_PAGE).contains(&args.per_page) {
        return error_result(format!("per_page must be between 1 and {MAX_PER_PAGE}"));
    }
    match with_db(db, |conn| {
        db::list_records(
            conn,
            &args.collection,
            args.page,
            args.per_page,
            &args.search,
        )
    }) {
        Ok(page) => json_result(&serde_json::to_value(page).unwrap_or(Value::Null)),
        Err(message) => error_result(format!("list records: {message}")),
    }
}

/// `blazers_records_get` returns one record by id.
pub fn handle_records_get(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: RecordArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    match with_db(db, |conn| db::get_record(conn, &args.collection, &args.id)) {
        Ok(Some(record)) => json_result(&serde_json::to_value(record).unwrap_or(Value::Null)),
        Ok(None) => error_result(format!("record '{}' not found", args.id)),
        Err(message) => error_result(format!("get record: {message}")),
    }
}

/// `blazers_records_create` inserts a record, generating an id when absent.
pub fn handle_records_create(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: CreateRecordArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    if let Err(message) = require_object(&args.data, "create record") {
        return error_result(message);
    }
    let id = args.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    match with_db(db, |conn| {
        db::insert_record(conn, &args.collection, &id, &args.data)
    }) {
        Ok(record) => json_result(&serde_json::to_value(record).unwrap_or(Value::Null)),
        Err(message) => error_result(format!("create record: {message}")),
    }
}

/// `blazers_records_update` replaces a record's data.
pub fn handle_records_update(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: UpdateRecordArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    if let Err(message) = require_object(&args.data, "update record") {
        return error_result(message);
    }
    match with_db(db, |conn| {
        db::update_record(conn, &args.collection, &args.id, &args.data)
    }) {
        Ok(record) => json_result(&serde_json::to_value(record).unwrap_or(Value::Null)),
        Err(message) => error_result(format!("update record: {message}")),
    }
}

/// `blazers_records_delete` removes one record.
pub fn handle_records_delete(db: &DbHandle, arguments: &str) -> ToolResult {
    let args: RecordArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    match with_db(db, |conn| {
        db::delete_record(conn, &args.collection, &args.id)
    }) {
        Ok(()) => json_result(&json!({ "id": args.id, "deleted": true })),
        Err(message) => error_result(format!("delete record: {message}")),
    }
}
