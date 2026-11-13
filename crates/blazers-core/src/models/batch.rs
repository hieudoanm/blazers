use serde::Deserialize;

/// One sub-request inside a `POST /api/batch` call (PocketBase parity).
#[derive(Debug, Deserialize)]
pub struct BatchItem {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct BatchRequest {
    #[serde(default)]
    pub requests: Vec<BatchItem>,
}
