use super::*;

/// GET /api/settings — list all settings (smtp_pass is masked).
pub async fn list_settings(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    crate::rbac::require_role_pub(&state, &headers, "*", "admin").await?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut map = db::list_settings(&conn)?;
    if map.contains_key("smtp_pass") {
        map.insert("smtp_pass".into(), Value::String("********".into()));
    }
    Ok(Json(Value::Object(map)))
}

/// PUT /api/settings — set one or more settings. `smtp_pass` is encrypted.
pub async fn update_settings(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<Value>,
) -> std::result::Result<Json<Value>, AppError> {
    crate::rbac::require_role_pub(&state, &headers, "*", "admin").await?;
    let obj = req
        .as_object()
        .ok_or_else(|| AppError::BadRequest("settings body must be an object".into()))?;
    if obj.is_empty() {
        return Err(AppError::BadRequest("no settings provided".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    for (key, value) in obj {
        let sval = match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        if key == "smtp_pass" {
            crate::mailer::set_password(&conn, &state.secrets_key, &sval)
                .map_err(AppError::Internal)?;
        } else {
            db::set_setting(&conn, key, &sval).map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    list_settings(headers, State(state)).await
}

/// DELETE /api/settings/{key}
pub async fn delete_setting(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    crate::rbac::require_role_pub(&state, &headers, "*", "admin").await?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !db::delete_setting(&conn, &key)? {
        return Err(AppError::NotFound("setting not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}
