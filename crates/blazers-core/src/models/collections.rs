use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub schema: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub data: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordsPage {
    pub records: Vec<Record>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    #[serde(default = "default_schema")]
    pub schema: String,
}

pub(super) fn default_schema() -> String {
    "{}".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CreateRecordRequest {
    pub id: Option<String>,
    #[serde(default = "default_data")]
    pub data: Value,
}

pub(super) fn default_data() -> Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecordRequest {
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RecordListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollectionRequest {
    pub schema: Option<String>,
}
