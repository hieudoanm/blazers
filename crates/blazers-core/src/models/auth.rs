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
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub mfa_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub verified: bool,
    #[serde(skip_serializing, default)]
    pub mfa_secret: String,
    #[serde(default)]
    pub mfa_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub current_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthWithOtpRequest {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub otp: String,
}

/// Optional record context for protected-file tokens: when set, the token is
/// only issued if the caller passes the record's `view_rule`.
#[derive(Debug, Deserialize)]
pub struct FileTokenRequest {
    #[serde(default)]
    pub collection: String,
    #[serde(default)]
    pub record: String,
}

/// PocketBase-style rule set for the managed `users` auth collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserRules {
    #[serde(default)]
    pub list_rule: String,
    #[serde(default)]
    pub view_rule: String,
    #[serde(default)]
    pub create_rule: String,
    #[serde(default)]
    pub update_rule: String,
    #[serde(default)]
    pub delete_rule: String,
}
