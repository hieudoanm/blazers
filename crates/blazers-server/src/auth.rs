use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use blazers_core::db;
use blazers_core::models::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".to_string())
}

pub fn register_user(conn: &Connection, email: &str, password: &str) -> Result<UserResponse> {
    if password.len() < 6 {
        return Err(AppError::BadRequest(
            "password must be at least 6 characters".into(),
        ));
    }
    let hashed =
        hash(password, DEFAULT_COST).map_err(|e| AppError::Internal(format!("hash: {e}")))?;
    let id = Uuid::new_v4().to_string().replace('-', "");
    let now = Utc::now().to_rfc3339();
    db::insert_user(conn, &id, email, &hashed)?;
    Ok(UserResponse {
        id,
        email: email.to_string(),
        data: serde_json::json!({}),
        verified: false,
        mfa_enabled: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Verifies email+password and returns the full user record (no token).
pub fn verify_credentials(conn: &Connection, email: &str, password: &str) -> Result<UserRecord> {
    let user = db::find_user_by_email(conn, email)?
        .and_then(|(id, _, _, _, _)| db::get_user(conn, &id).ok().flatten())
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;
    if !verify(password, &user.password).unwrap_or(false) {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }
    Ok(user)
}

pub fn login_user(conn: &Connection, email: &str, password: &str) -> Result<LoginResponse> {
    let user = verify_credentials(conn, email, password)?;
    let token = issue_token(&user.id, &user.email)?;
    Ok(LoginResponse {
        user: user_response(&user),
        token,
    })
}

/// Issues a standard user JWT (login response token). Used by login, OTP,
/// OAuth and impersonation flows.
pub fn issue_token(user_id: &str, email: &str) -> Result<String> {
    generate_token(user_id, email).map_err(|e| AppError::Internal(format!("generate token: {e}")))
}

fn generate_token(
    user_id: &str,
    email: &str,
) -> std::result::Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        user_id: user_id.to_string(),
        email: email.to_string(),
        exp: now + 72 * 3600,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
}

pub fn validate_token(token_str: &str) -> std::result::Result<Claims, AppError> {
    let token = decode::<Claims>(
        token_str,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("invalid token".into()))?;
    Ok(token.claims)
}

/// Short-lived token granting access to a single protected file.
/// Format: `file:<file_id>` embedded in the JWT subject claim.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTokenClaims {
    pub file_id: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn generate_file_token(file_id: &str, ttl_secs: usize) -> String {
    let now = Utc::now().timestamp() as usize;
    let claims = FileTokenClaims {
        file_id: file_id.to_string(),
        exp: now + ttl_secs,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .unwrap_or_default()
}

/// Validates a file token and returns the file id it was issued for.
pub fn validate_file_token(token_str: &str) -> std::result::Result<String, AppError> {
    let token = decode::<FileTokenClaims>(
        token_str,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("invalid file token".into()))?;
    Ok(token.claims.file_id)
}

/// Serializes a user record for API responses (no password).
pub fn user_response(u: &UserRecord) -> UserResponse {
    UserResponse {
        id: u.id.clone(),
        email: u.email.clone(),
        data: u.data.clone(),
        verified: u.verified,
        mfa_enabled: u.mfa_enabled,
        created_at: u.created_at.clone(),
        updated_at: u.updated_at.clone(),
    }
}

#[cfg(test)]
mod tests;
