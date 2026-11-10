use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WSConnection {
    pub id: String,
    pub remote_addr: String,
    pub path: String,
    pub user_agent: String,
    pub connected_at: String,
    pub disconnected_at: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WSMessage {
    pub id: String,
    pub connection_id: String,
    pub direction: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WSSendRequest {
    pub content: String,
}
