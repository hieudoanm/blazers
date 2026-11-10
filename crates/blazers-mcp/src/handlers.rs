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

pub mod data;
pub mod io;
pub mod kv;

pub use data::*;
pub use io::*;
pub use kv::*;

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
