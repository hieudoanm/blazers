use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub scope: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub scope: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSecretRequest {
    pub name: Option<String>,
    pub value: Option<String>,
    pub scope: Option<String>,
}
