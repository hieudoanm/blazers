use super::*;

pub async fn list_webhooks(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Webhook>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let hooks = db::list_webhooks(&conn)?;
    Ok(Json(hooks))
}

pub async fn create_webhook(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateWebhookRequest>,
) -> std::result::Result<Json<Webhook>, AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if req.url.is_empty() {
        return Err(AppError::BadRequest("url is required".into()));
    }
    if req.events.is_empty() {
        return Err(AppError::BadRequest(
            "at least one event is required".into(),
        ));
    }
    if let Err(e) = webhook::validate_events(&req.events) {
        return Err(AppError::BadRequest(e));
    }
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let h = db::insert_webhook(&conn, &id, &req.name, &req.url, &req.events, &req.secret)?;
    Ok(Json(h))
}

pub async fn get_webhook(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Webhook>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let h = db::get_webhook(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(h))
}

pub async fn update_webhook(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWebhookRequest>,
) -> std::result::Result<Json<Webhook>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing =
        db::get_webhook(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let url = req.url.unwrap_or(existing.url);
    let events = req.events.unwrap_or(existing.events);
    let secret = req.secret.unwrap_or(existing.secret);
    let is_active = req.is_active.unwrap_or(existing.is_active);

    if let Err(e) = webhook::validate_events(&events) {
        return Err(AppError::BadRequest(e));
    }

    let h = db::update_webhook(&conn, &id, &name, &url, &events, &secret, is_active)?;
    Ok(Json(h))
}

pub async fn delete_webhook(
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
    let existing = db::get_webhook(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    db::delete_webhook(&conn, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_webhook_logs(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<WebhookLog>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_webhook(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let logs = db::list_webhook_logs(&conn, &id)?;
    Ok(Json(logs))
}
