//! Passwordless OTP auth: `request-otp` emails a 6-digit code,
//! `auth-with-otp` exchanges it for a JWT (PocketBase `authWithOTP` parity).

use super::*;
use crate::auth;

fn gen_otp() -> String {
    format!("{:06}", rand::random_range(0..1_000_000u32))
}

async fn login_response_for(
    state: &AppState,
    user_id: &str,
) -> std::result::Result<Json<blazers_core::models::LoginResponse>, AppError> {
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user =
        db::get_user(&conn, user_id)?.ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let token = auth::issue_token(&user.id, &user.email)?;
    Ok(Json(blazers_core::models::LoginResponse {
        user: auth::user_response(&user),
        token,
    }))
}

/// POST /api/auth/request-otp — sends a one-time code to the email.
/// Always returns success for unknown emails (no enumeration).
pub async fn request_otp(
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
    if let Some((id, _, _, _, _)) = db::find_user_by_email(&conn, &req.email)? {
        let code = gen_otp();
        db::set_user_otp(&conn, &id, &code, 900)?; // 15 minutes
        let body = format!("Your Blazers sign-in code is: {code}\n\nIt expires in 15 minutes.");
        let _ = crate::mailer::send(&state, &req.email, "Blazers sign-in code", &body).await;
        let mut out = serde_json::json!({ "message": "code sent" });
        if !crate::mailer::is_configured(&conn) {
            out["dev_otp"] = serde_json::json!(code);
        }
        return Ok(Json(out));
    }
    Ok(Json(serde_json::json!({ "message": "code sent" })))
}

/// POST /api/auth/auth-with-otp — exchanges email + code for a JWT.
pub async fn auth_with_otp(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<AuthWithOtpRequest>,
) -> std::result::Result<Json<blazers_core::models::LoginResponse>, AppError> {
    if req.email.is_empty() || req.otp.len() != 6 {
        return Err(AppError::BadRequest(
            "email and 6-digit otp are required".into(),
        ));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user_id = db::find_user_by_otp(&conn, &req.email, &req.otp)?
        .ok_or_else(|| AppError::Unauthorized("invalid or expired code".into()))?;
    db::clear_user_otp(&conn, &user_id)?;
    drop(conn);
    login_response_for(&state, &user_id).await
}
