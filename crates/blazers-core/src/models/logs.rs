use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub level: String,
    pub message: String,
    #[serde(default)]
    pub meta: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub meta: String,
}
