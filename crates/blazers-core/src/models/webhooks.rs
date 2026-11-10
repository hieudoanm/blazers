use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    #[serde(default)]
    pub secret: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookLog {
    pub id: String,
    pub webhook_id: String,
    pub event: String,
    pub url: String,
    pub request_body: String,
    pub response_status: i64,
    pub response_body: String,
    #[serde(default)]
    pub error: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub secret: Option<String>,
    pub is_active: Option<bool>,
}
