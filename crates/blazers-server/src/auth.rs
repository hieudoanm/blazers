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
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn login_user(conn: &Connection, email: &str, password: &str) -> Result<LoginResponse> {
    let user = db::find_user_by_email(conn, email)?;
    match user {
        Some((id, email, created_at, updated_at, hashed_pwd)) => {
            if !verify(password, &hashed_pwd).unwrap_or(false) {
                return Err(AppError::Unauthorized("invalid credentials".into()));
            }
            let token = generate_token(&id, &email)
                .map_err(|e| AppError::Internal(format!("generate token: {e}")))?;
            Ok(LoginResponse {
                user: UserResponse {
                    id,
                    email,
                    created_at,
                    updated_at,
                },
                token,
            })
        }
        None => Err(AppError::Unauthorized("invalid credentials".into())),
    }
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

#[cfg(test)]
mod tests;
