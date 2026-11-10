use super::*;

pub async fn handle_pubsub_topics_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<PubSubTopic>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let topics = db::list_pubsub_topics(&conn)?;
    Ok(Json(topics))
}

pub async fn handle_pubsub_topics_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreatePubSubTopicRequest>,
) -> std::result::Result<(StatusCode, Json<PubSubTopic>), AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let existing = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_pubsub_topic_by_name(&conn, &req.name)?
    };
    if existing.is_some() {
        return Err(AppError::Conflict("topic already exists".into()));
    }
    let id = Uuid::new_v4().to_string().replace('-', "");
    let topic = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_pubsub_topic(&conn, &id, &req.name)?
    };
    webhook::dispatch_event(
        &state,
        webhook::EVENT_PUBSUB_TOPIC_CREATE,
        serde_json::json!({"topic": &topic}),
    );
    Ok((StatusCode::CREATED, Json(topic)))
}

pub async fn handle_pubsub_topics_get(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Json<PubSubTopic>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let topic = db::get_pubsub_topic_by_name(&conn, &name)?
        .ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(topic))
}

pub async fn handle_pubsub_topics_delete(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let existing = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_pubsub_topic_by_name(&conn, &name)?
    };
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::delete_pubsub_topic(&conn, &name)?;
    }
    if let Some(topic) = existing {
        webhook::dispatch_event(
            &state,
            webhook::EVENT_PUBSUB_TOPIC_DELETE,
            serde_json::json!({"topic": &topic}),
        );
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn handle_pubsub_messages_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Json<Vec<PubSubMessage>>, AppError> {
    extract_claims(&headers)?;
    let (topic_id,) = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let topic = db::get_pubsub_topic_by_name(&conn, &name)?
            .ok_or_else(|| AppError::NotFound("topic not found".into()))?;
        (topic.id,)
    };
    let msg_list = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::list_pubsub_messages(&conn, &topic_id)?
    };
    Ok(Json(msg_list))
}

pub async fn handle_pubsub_messages_create(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<CreatePubSubMessageRequest>,
) -> std::result::Result<(StatusCode, Json<PubSubMessage>), AppError> {
    extract_claims(&headers)?;
    if req.body.is_empty() {
        return Err(AppError::BadRequest("body is required".into()));
    }
    let topic = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_pubsub_topic_by_name(&conn, &name)?
            .ok_or_else(|| AppError::NotFound("topic not found".into()))?
    };
    let id = Uuid::new_v4().to_string().replace('-', "");
    let msg = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::insert_pubsub_message(&conn, &id, &topic.id, &req.body)?
    };
    pubsub::broadcast_pubsub(&state, &msg).await;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_PUBSUB_MESSAGE_CREATE,
        serde_json::json!({"message": &msg}),
    );
    Ok((StatusCode::CREATED, Json(msg)))
}

// --- Cron Jobs ---
