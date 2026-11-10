//! Tool handlers backed by the real Blazers database.
//!
//! Every handler here performs actual work against SQLite. The Go port
//! advertises `blazers_export` and `blazers_import` but answers with
//! "not implemented"; this port wires them to the same code paths the HTTP
//! `/export` and `/import` routes use, so a model calling them gets real data.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde_json::{Value, json};

use super::protocol::{ToolResult, error_result, text_result};
use blazers_core::db;
use blazers_core::models::AppError;
use blazers_server::import_export::ExportData;

/// A database handle shared by every handler in one server.
pub type DbHandle = Arc<Mutex<Connection>>;

/// Opens the same database the HTTP server uses, migrating it if needed.
pub fn open_db() -> std::io::Result<DbHandle> {
    let path = db::data_dir().join("data.db");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)
        .map_err(|err| std::io::Error::other(format!("open database: {err}")))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|err| std::io::Error::other(format!("configure database: {err}")))?;
    db::migrate_db(&conn).map_err(|err| std::io::Error::other(format!("migrate: {err}")))?;
    Ok(Arc::new(Mutex::new(conn)))
}

/// Runs `body` while holding the database lock, mapping a database error to a
/// tool failure. A handler returns an errored result rather than panicking, so a
/// model can correct the call and the session survives.
fn with_db<T>(
    db: &DbHandle,
    body: impl FnOnce(&Connection) -> Result<T, AppError>,
) -> Result<T, String> {
    let conn = db
        .lock()
        .map_err(|_| "the database lock is poisoned".to_string())?;
    body(&conn).map_err(|err| err.to_string())
}

/// Renders a value as indented JSON, or an errored result when it will not
/// encode. A silently empty result would read to a model as a successful call.
fn json_result(value: &Value) -> ToolResult {
    match serde_json::to_string_pretty(value) {
        Ok(encoded) => text_result(encoded),
        Err(err) => error_result(format!("encode result: {err}")),
    }
}

/// Parses handler arguments, treating absent or null arguments as an empty
/// object so a no-argument tool needs no special case.
pub fn parse_args<T: serde::de::DeserializeOwned>(arguments: &str) -> Result<T, String> {
    serde_json::from_str(arguments).map_err(|err| format!("parse args: {err}"))
}

/// Arguments of `blazers_collections_create`.
#[derive(Debug, serde::Deserialize)]
pub struct CreateCollectionArgs {
    pub name: String,
    #[serde(default)]
    pub schema: String,
}

/// Arguments of `blazers_collections_delete` and `blazers_records_delete`.
#[derive(Debug, serde::Deserialize)]
pub struct NameArgs {
    pub name: String,
}

/// Arguments of `blazers_records_get` and `blazers_records_delete`.
#[derive(Debug, serde::Deserialize)]
pub struct RecordArgs {
    pub collection: String,
    pub id: String,
}

/// Arguments of `blazers_records_list`.
#[derive(Debug, serde::Deserialize)]
pub struct ListRecordsArgs {
    pub collection: String,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    #[serde(default)]
    pub search: String,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    50
}

/// Arguments of `blazers_records_create`.
#[derive(Debug, serde::Deserialize)]
pub struct CreateRecordArgs {
    pub collection: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub data: Value,
}

/// Record bodies are stored verbatim, so a non-object would silently persist a
/// JSON string where the caller meant an object.
fn require_object(data: &Value, tool: &str) -> Result<(), String> {
    if data.is_object() {
        Ok(())
    } else {
        Err(format!(
            "{tool}: data is required and must be a JSON object"
        ))
    }
}

/// Arguments of `blazers_records_update`.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateRecordArgs {
    pub collection: String,
    pub id: String,
    #[serde(default)]
    pub data: Value,
}

/// Arguments of `blazers_export`.
#[derive(Debug, serde::Deserialize)]
pub struct ExportArgs {
    /// Accepted for parity with the Go port. JSON is the only format this
    /// server produces, and the value is reported back so a caller asking for
    /// CSV learns that rather than silently receiving JSON.
    pub format: String,
}

/// Arguments of `blazers_import`.
#[derive(Debug, serde::Deserialize)]
pub struct ImportArgs {
    pub format: String,
    pub data: String,
}

/// Caps a single page so a model cannot request the whole table at once.
const MAX_PER_PAGE: i64 = 200;

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
