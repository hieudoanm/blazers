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
mod tests {
    use super::*;

    static JWT_ENV_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

    #[test]
    fn jwt_secret_returns_default_when_env_var_not_set() {
        let _lock = JWT_ENV_LOCK.lock().unwrap();
        let saved = std::env::var("JWT_SECRET");
        unsafe { std::env::remove_var("JWT_SECRET") };
        assert_eq!(jwt_secret(), "dev-secret-change-in-production");
        if let Ok(v) = saved {
            unsafe { std::env::set_var("JWT_SECRET", v) };
        }
    }

    #[test]
    fn jwt_secret_returns_env_var_when_set() {
        let _lock = JWT_ENV_LOCK.lock().unwrap();
        let saved = std::env::var("JWT_SECRET");
        unsafe { std::env::set_var("JWT_SECRET", "test-secret-123") };
        assert_eq!(jwt_secret(), "test-secret-123");
        match saved {
            Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
            Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
        }
    }

    #[test]
    fn round_trip_generate_and_validate() {
        let _lock = JWT_ENV_LOCK.lock().unwrap();
        let saved = std::env::var("JWT_SECRET");
        unsafe { std::env::set_var("JWT_SECRET", "dev-secret-change-in-production") };
        let token = generate_token("user-1", "test@example.com").unwrap();
        let claims = validate_token(&token).unwrap();
        match saved {
            Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
            Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
        }
        assert_eq!(claims.user_id, "user-1");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn invalid_token_format_returns_unauthorized() {
        let result = validate_token("not-a-jwt-token");
        match result.unwrap_err() {
            AppError::Unauthorized(_) => {}
            _ => panic!("expected Unauthorized"),
        }
    }

    #[test]
    fn token_with_wrong_secret_returns_error() {
        let _lock = JWT_ENV_LOCK.lock().unwrap();
        let saved = std::env::var("JWT_SECRET");
        unsafe { std::env::set_var("JWT_SECRET", "secret-a") };
        let token = generate_token("user-1", "test@example.com").unwrap();
        unsafe { std::env::set_var("JWT_SECRET", "secret-b") };
        let result = validate_token(&token);
        match saved {
            Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
            Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
        }
        assert!(result.is_err());
    }

    #[test]
    fn generate_token_with_empty_fields() {
        let _lock = JWT_ENV_LOCK.lock().unwrap();
        let saved = std::env::var("JWT_SECRET");
        unsafe { std::env::set_var("JWT_SECRET", "dev-secret-change-in-production") };
        let token = generate_token("", "").unwrap();
        let claims = validate_token(&token).unwrap();
        match saved {
            Ok(v) => unsafe { std::env::set_var("JWT_SECRET", v) },
            Err(_) => unsafe { std::env::remove_var("JWT_SECRET") },
        }
        assert_eq!(claims.user_id, "");
        assert_eq!(claims.email, "");
    }
}
