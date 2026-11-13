use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::*;

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

/// Counts of rows written by `db::import_all`.
#[derive(Debug, Default, Serialize)]
pub struct TransferSummary {
    pub collections: i64,
    pub records: i64,
    pub buckets: i64,
    pub files: i64,
}
