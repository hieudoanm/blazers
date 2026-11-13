//! TOTP multi-factor auth: enable/confirm/disable plus the login challenge.
//! Secret lives in `_users.mfa_secret` (never serialized); `mfa_enabled`
//! flips only after a valid code confirms enrollment.

use axum::{Json, extract::State, http::HeaderMap};
use serde::Deserialize;
use serde_json::Value;
use totp_rs::{Builder, Secret, Totp};

use super::*;
use crate::auth::{self, Claims};

fn totp_for(secret_b32: &str) -> std::result::Result<Totp, AppError> {
    let secret = Secret::try_from_base32(secret_b32)
        .map_err(|e| AppError::Internal(format!("mfa secret: {e}")))?;
    Builder::new()
        .with_secret(secret)
        .build()
        .map_err(|e| AppError::Internal(format!("mfa totp: {e}")))
}

fn check_code(secret_b32: &str, _email: &str, code: &str) -> bool {
    totp_for(secret_b32)
        .map(|t| t.check_current(code).is_some())
        .unwrap_or(false)
}

async fn load_user(state: &AppState, claims: &Claims) -> std::result::Result<UserRecord, AppError> {
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::get_user(&conn, &claims.user_id)?.ok_or_else(|| AppError::NotFound("user not found".into()))
}

/// POST /api/auth/mfa/enable — generates a TOTP secret (auth required).
/// Returns the base32 secret and otpauth URL; MFA stays disabled until
/// `/mfa/confirm` verifies a code.
pub async fn mfa_enable(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let user = load_user(&state, &claims).await?;
    if user.mfa_enabled {
        return Err(AppError::BadRequest("mfa already enabled".into()));
    }
    let b32 = Secret::generate().to_base32();
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::set_user_mfa(&conn, &user.id, Some(&b32), false)?;
    Ok(Json(serde_json::json!({
        "secret": b32,
        "otpauth_url": format!("otpauth://totp/blazers:{}?secret={b32}&issuer=blazers", user.email),
    })))
}

#[derive(Deserialize)]
pub struct MfaCodeRequest {
    #[serde(default)]
    pub code: String,
}

/// POST /api/auth/mfa/confirm — verifies a TOTP code and enables MFA.
pub async fn mfa_confirm(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<MfaCodeRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let user = load_user(&state, &claims).await?;
    if user.mfa_secret.is_empty() {
        return Err(AppError::BadRequest("call /mfa/enable first".into()));
    }
    if !check_code(&user.mfa_secret, &user.email, &req.code) {
        return Err(AppError::Unauthorized("invalid code".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::set_user_mfa(&conn, &user.id, Some(&user.mfa_secret), true)?;
    Ok(Json(serde_json::json!({ "message": "mfa enabled" })))
}

/// POST /api/auth/mfa/disable — requires a valid TOTP code.
pub async fn mfa_disable(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<MfaCodeRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let user = load_user(&state, &claims).await?;
    if !user.mfa_enabled || user.mfa_secret.is_empty() {
        return Err(AppError::BadRequest("mfa is not enabled".into()));
    }
    if !check_code(&user.mfa_secret, &user.email, &req.code) {
        return Err(AppError::Unauthorized("invalid code".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::set_user_mfa(&conn, &user.id, None, false)?;
    Ok(Json(serde_json::json!({ "message": "mfa disabled" })))
}

#[derive(Deserialize)]
pub struct MfaLoginRequest {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub code: String,
}

/// POST /api/auth/mfa/login — password + TOTP code → JWT.
pub async fn mfa_login(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<MfaLoginRequest>,
) -> std::result::Result<Json<blazers_core::models::LoginResponse>, AppError> {
    if req.email.is_empty() || req.password.is_empty() || req.code.is_empty() {
        return Err(AppError::BadRequest(
            "email, password and code are required".into(),
        ));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = auth::verify_credentials(&conn, &req.email, &req.password)?;
    if !user.mfa_enabled || user.mfa_secret.is_empty() {
        return Err(AppError::BadRequest(
            "mfa is not enabled for this user".into(),
        ));
    }
    if !check_code(&user.mfa_secret, &user.email, &req.code) {
        return Err(AppError::Unauthorized("invalid mfa code".into()));
    }
    let token = auth::issue_token(&user.id, &user.email)?;
    Ok(Json(blazers_core::models::LoginResponse {
        user: auth::user_response(&user),
        token,
    }))
}
