use super::*;

pub async fn list_ws_connections(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<WSConnection>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let connections = db::list_ws_connections(&conn)?;
    Ok(Json(connections))
}

pub async fn get_ws_connection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<WSConnection>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let connection =
        db::get_ws_connection(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(connection))
}

pub async fn delete_ws_connection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let _ = websocket::close_client(&state.ws_hub, &id).await;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_ws_connection(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    db::delete_ws_connection(&conn, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn broadcast_ws_message(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<WSSendRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let content = req.content;

    let ids: Vec<String> = {
        let hub = state.ws_hub.read().await;
        hub.clients.keys().cloned().collect()
    };

    for id in &ids {
        let sender = {
            let hub = state.ws_hub.read().await;
            hub.clients.get(id).map(|c| c.sender.clone())
        };
        if let Some(sender) = sender {
            let msg = Message::Text(content.clone().into());
            sender.send(msg).ok();
        }
        if let Ok(conn) = state.db.get().await {
            db::insert_ws_message(&conn, id, "sent", &content).ok();
        }
    }

    Ok(Json(serde_json::json!({"sent": ids.len()})))
}

pub async fn send_ws_message(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<WSSendRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let content = req.content;

    let sent =
        websocket::send_to_client(&state.ws_hub, &id, Message::Text(content.clone().into())).await;
    if let Ok(conn) = state.db.get().await {
        db::insert_ws_message(&conn, &id, "sent", &content).ok();
    }

    if sent {
        Ok(Json(serde_json::json!({"sent": true})))
    } else {
        Err(AppError::NotFound("connection not found".into()))
    }
}

pub async fn list_ws_messages(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<WSMessage>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_ws_connection(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let messages = db::list_ws_messages(&conn, &id)?;
    Ok(Json(messages))
}

pub async fn list_all_ws_messages(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<WSMessage>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let messages = db::list_all_ws_messages(&conn)?;
    Ok(Json(messages))
}

// --- Cache ---
