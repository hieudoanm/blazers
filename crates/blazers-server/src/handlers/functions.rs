//! Edge functions: JS source stored in `_functions`, executed by the
//! embedded QuickJS runtime (`crate::js`). CRUD is admin-only; invocation
//! is open but receives the caller's auth context when a token is sent.

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde_json::{Value, json};

use super::*;
use crate::auth;

async fn conn(
    state: &AppState,
) -> std::result::Result<deadpool::managed::Object<db::ConnectionManager>, AppError> {
    state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

async fn require_admin(state: &AppState, headers: &HeaderMap) -> std::result::Result<(), AppError> {
    let claims = extract_claims(headers)?;
    let conn = conn(state).await?;
    rbac::require_role(&conn, &claims.user_id, "*", "admin")
}

/// GET /api/functions — list stored functions (admin).
pub async fn list_functions(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Function>>, AppError> {
    require_admin(&state, &headers).await?;
    let conn = conn(&state).await?;
    Ok(Json(db::list_functions(&conn)?))
}

/// POST /api/functions — create or replace a function (admin).
pub async fn upsert_function(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<UpsertFunctionRequest>,
) -> std::result::Result<Json<Function>, AppError> {
    require_admin(&state, &headers).await?;
    if req.name.is_empty() || req.code.trim().is_empty() {
        return Err(AppError::BadRequest("name and code are required".into()));
    }
    let conn = conn(&state).await?;
    Ok(Json(db::upsert_function(&conn, &req.name, &req.code)?))
}

/// DELETE /api/functions/{name} — remove a function (admin).
pub async fn delete_function(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    require_admin(&state, &headers).await?;
    let conn = conn(&state).await?;
    if !db::delete_function(&conn, &name)? {
        return Err(AppError::NotFound("function not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/functions/{name} — invoke a function.
/// Body: `{ "input": <any JSON> }`. The function's `handler`/`main` receives
/// `{ "input": ..., "auth": { "user_id", "email" } | null }` and its return
/// value is wrapped as `{ "result": ... }`. Auth is optional — send a Bearer
/// token to expose the caller's identity to the function.
pub async fn invoke_function(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<InvokeFunctionRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    let conn = conn(&state).await?;
    let func = db::get_function(&conn, &name)?
        .ok_or_else(|| AppError::NotFound("function not found".into()))?;

    let auth_ctx = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .and_then(|t| auth::validate_token(t).ok())
        .map(|c| json!({ "user_id": c.user_id, "email": c.email }));

    let input = json!({ "input": req.input, "auth": auth_ctx });
    let code = func.code.clone();
    let result = tokio::task::spawn_blocking(move || crate::js::run(&code, &input))
        .await
        .map_err(|e| AppError::Internal(format!("function task: {e}")))??;
    Ok(Json(json!({ "result": result })))
}
