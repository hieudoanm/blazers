use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub ttl: i64,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SetCacheRequest {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
    pub ttl: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CacheIncrRequest {
    #[serde(default)]
    pub by: i64,
}

#[derive(Debug, Deserialize)]
pub struct CacheAppendRequest {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct CacheExpireRequest {
    #[serde(default)]
    pub ttl: i64,
}

#[derive(Debug, Deserialize)]
pub struct CacheListPushRequest {
    #[serde(default)]
    pub values: Vec<serde_json::Value>,
    #[serde(default)]
    pub left: bool,
}

#[derive(Debug, Deserialize)]
pub struct CacheListPopRequest {
    #[serde(default)]
    pub left: bool,
    pub count: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CacheHashSetRequest {
    pub field: Option<String>,
    pub value: Option<serde_json::Value>,
    pub fields: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteCacheKeysRequest {
    #[serde(default)]
    pub keys: Vec<String>,
}
