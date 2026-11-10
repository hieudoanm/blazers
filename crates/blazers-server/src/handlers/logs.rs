use super::*;

pub async fn handle_logs_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<LogEntry>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let logs = db::list_logs(&conn)?;
    Ok(Json(logs))
}

pub async fn handle_logs_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateLogRequest>,
) -> std::result::Result<Json<LogEntry>, AppError> {
    extract_claims(&headers)?;

    if req.message.is_empty() {
        return Err(AppError::BadRequest("message is required".into()));
    }
    let level = if req.level.is_empty() {
        "info".to_string()
    } else {
        req.level.clone()
    };
    if !crate::log::validate_log_level(&level) {
        return Err(AppError::BadRequest(
            "level must be debug, info, warn, or error".into(),
        ));
    }
    let meta = if req.meta.is_empty() {
        "{}".to_string()
    } else {
        req.meta.clone()
    };

    let id = Uuid::new_v4().to_string().replace('-', "");
    let entry = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_log(&conn, &id, &level, &req.message, &meta)?
    };

    crate::log::broadcast_log(&state, &entry).await;

    webhook::dispatch_event(
        &state,
        webhook::EVENT_LOG_CREATE,
        serde_json::to_value(&entry).unwrap_or_default(),
    );

    Ok(Json(entry))
}

pub async fn handle_logs_clear(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::clear_logs(&conn)?;
    Ok(Json(serde_json::json!({"status": "cleared"})))
}

// PubSub handlers
