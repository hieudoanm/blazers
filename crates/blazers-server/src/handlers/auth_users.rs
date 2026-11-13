use super::*;
use crate::auth::Claims;

pub(crate) fn user_json(u: &UserRecord) -> Value {
    serde_json::json!({
        "id": u.id,
        "email": u.email,
        "data": u.data,
        "verified": u.verified,
        "mfa_enabled": u.mfa_enabled,
        "created_at": u.created_at,
        "updated_at": u.updated_at,
    })
}

async fn load_user(state: &AppState, claims: &Claims) -> std::result::Result<UserRecord, AppError> {
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::get_user(&conn, &claims.user_id)?.ok_or_else(|| AppError::NotFound("user not found".into()))
}

/// GET /api/auth/me — current user profile.
pub async fn get_me(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let user = load_user(&state, &claims).await?;
    Ok(Json(user_json(&user)))
}

/// PATCH /api/auth/me — update custom `data` fields and/or password.
pub async fn update_me(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<UpdateUserRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = db::get_user(&conn, &claims.user_id)?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    if let Some(new_password) = req.password.as_deref() {
        if new_password.len() < 6 {
            return Err(AppError::BadRequest(
                "password must be at least 6 characters".into(),
            ));
        }
        // Require the current password when one is set (OAuth-only accounts
        // have an empty stored password and can set one directly).
        if !user.password.is_empty()
            && !bcrypt::verify(
                req.current_password.as_deref().unwrap_or(""),
                &user.password,
            )
            .unwrap_or(false)
        {
            return Err(AppError::Unauthorized("current password required".into()));
        }
        let hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("hash: {e}")))?;
        db::update_user_password(&conn, &user.id, &hash)?;
    }

    if let Some(data) = req.data {
        db::update_user_data(&conn, &user.id, &data)?;
    }

    let updated = db::get_user(&conn, &user.id)?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    Ok(Json(user_json(&updated)))
}

/// POST /api/auth/forgot-password — emails a reset token.
/// Always returns 200 (does not reveal whether the email exists).
pub async fn forgot_password(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<EmailRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    if req.email.is_empty() {
        return Err(AppError::BadRequest("email is required".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let found = db::find_user_by_email(&conn, &req.email)?;
    if let Some((id, _, _, _, _)) = found {
        let token = db::new_token();
        db::set_user_token(&conn, &id, "reset", &token, 3600)?;
        let base = crate::mailer::app_url(&conn);
        let link = format!("{base}/reset-password?token={token}");
        let body =
            format!("Reset your Blazers password:\n\n{link}\n\nThis link expires in 1 hour.");
        let _ = crate::mailer::send(&state, &req.email, "Blazers password reset", &body).await;
        // Dev convenience: expose the token when SMTP is not configured.
        if !crate::mailer::is_configured(&conn) {
            return Ok(Json(serde_json::json!({
                "message": "reset email sent",
                "dev_token": token,
            })));
        }
    }
    Ok(Json(serde_json::json!({ "message": "reset email sent" })))
}

/// POST /api/auth/reset-password — exchanges a reset token for a new password.
pub async fn reset_password(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<ResetPasswordRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    if req.token.is_empty() || req.password.len() < 6 {
        return Err(AppError::BadRequest(
            "token and a password of at least 6 characters are required".into(),
        ));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user_id = db::find_user_by_token(&conn, "reset", &req.token)?
        .ok_or_else(|| AppError::BadRequest("invalid or expired token".into()))?;
    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("hash: {e}")))?;
    db::update_user_password(&conn, &user_id, &hash)?;
    db::clear_user_token(&conn, &user_id, "reset")?;
    Ok(Json(serde_json::json!({ "message": "password updated" })))
}

/// POST /api/auth/verify-email — sends a verification email.
/// Requires auth; rate of sending is not limited beyond auth.
pub async fn request_verification(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = db::get_user(&conn, &claims.user_id)?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    if user.verified {
        return Ok(Json(serde_json::json!({ "message": "already verified" })));
    }
    let token = db::new_token();
    db::set_user_token(&conn, &user.id, "verify", &token, 86400)?;
    let base = crate::mailer::app_url(&conn);
    let link = format!("{base}/verify-email?token={token}");
    let body = format!("Verify your Blazers email:\n\n{link}\n\nThis link expires in 24 hours.");
    let _ = crate::mailer::send(&state, &user.email, "Blazers email verification", &body).await;
    let mut out = serde_json::json!({ "message": "verification email sent" });
    if !crate::mailer::is_configured(&conn) {
        out["dev_token"] = serde_json::json!(token);
    }
    Ok(Json(out))
}

/// POST /api/auth/verify-email/confirm — exchanges a verify token.
pub async fn confirm_verification(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<TokenRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    if req.token.is_empty() {
        return Err(AppError::BadRequest("token is required".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user_id = db::find_user_by_token(&conn, "verify", &req.token)?
        .ok_or_else(|| AppError::BadRequest("invalid or expired token".into()))?;
    db::set_user_verified(&conn, &user_id, true)?;
    db::clear_user_token(&conn, &user_id, "verify")?;
    Ok(Json(serde_json::json!({ "message": "email verified" })))
}
