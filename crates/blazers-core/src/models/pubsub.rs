use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubTopic {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubMessage {
    pub id: String,
    pub topic_id: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePubSubTopicRequest {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePubSubMessageRequest {
    #[serde(default)]
    pub body: String,
}
