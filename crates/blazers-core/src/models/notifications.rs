use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(rename = "type")]
    pub ntype: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(rename = "type")]
    pub ntype: String,
}
