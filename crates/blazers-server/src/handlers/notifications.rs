use super::*;

pub async fn handle_notifications_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Notification>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n = db::list_notifications(&conn)?;
    Ok(Json(n))
}

pub async fn handle_notifications_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateNotificationRequest>,
) -> std::result::Result<Json<Notification>, AppError> {
    extract_claims(&headers)?;

    if req.title.is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let ntype = if req.ntype.is_empty() {
        "info".to_string()
    } else {
        req.ntype.clone()
    };
    if !notification::validate_notification_type(&ntype) {
        return Err(AppError::BadRequest(
            "type must be info, success, warning, or error".into(),
        ));
    }

    let id = Uuid::new_v4().to_string().replace('-', "");
    let notification_entry = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_notification(&conn, &id, &req.title, &req.body, &ntype)?
    };

    notification::broadcast_notification(&state, &notification_entry).await;

    webhook::dispatch_event(
        &state,
        webhook::EVENT_NOTIFICATION_CREATE,
        serde_json::to_value(&notification_entry).unwrap_or_default(),
    );

    Ok(Json(notification_entry))
}

pub async fn handle_notifications_get(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Notification>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n =
        db::get_notification(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(n))
}

pub async fn handle_notifications_mark_read(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Notification>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n = db::update_notification_read(&conn, &id)?;
    Ok(Json(n))
}

pub async fn handle_notifications_delete(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let ok = db::delete_notification(&conn, &id)?;
    if !ok {
        return Err(AppError::NotFound("not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_notifications_clear(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::clear_notifications(&conn)?;
    Ok(Json(serde_json::json!({"status": "cleared"})))
}
