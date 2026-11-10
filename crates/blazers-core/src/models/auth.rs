use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
}
