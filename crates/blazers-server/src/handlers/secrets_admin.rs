use super::*;

#[derive(Serialize)]
pub struct SecretListItem {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn list_secrets(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<SecretListItem>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let secrets = db::list_secrets(&conn)?;
    let items: Vec<SecretListItem> = secrets
        .into_iter()
        .map(|s| SecretListItem {
            id: s.id,
            name: s.name,
            scope: s.scope,
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
        .collect();
    Ok(Json(items))
}

pub async fn create_secret(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateSecretRequest>,
) -> std::result::Result<(StatusCode, Json<Secret>), AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let scope = if req.scope.is_empty() {
        "general".to_string()
    } else {
        req.scope
    };
    let encrypted =
        secrets::encrypt_secret(&state.secrets_key, &req.value).map_err(AppError::Internal)?;
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let secret = db::insert_secret(&conn, &id, &req.name, &encrypted, &scope)?;
    webhook::dispatch_event(
        &state,
        secrets::EVENT_SECRET_CREATE,
        secrets::webhook_secret_data(&secret),
    );
    Ok((StatusCode::CREATED, Json(secret)))
}

pub async fn get_secret(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Secret>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut secret =
        db::get_secret(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    secret.value =
        secrets::decrypt_secret(&state.secrets_key, &secret.value).map_err(AppError::Internal)?;
    Ok(Json(secret))
}

pub async fn update_secret(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSecretRequest>,
) -> std::result::Result<Json<Secret>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing =
        db::get_secret(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let scope = req.scope.unwrap_or(existing.scope);
    let plaintext = match req.value {
        Some(v) if !v.is_empty() => v,
        _ => secrets::decrypt_secret(&state.secrets_key, &existing.value).unwrap_or_default(),
    };
    let encrypted =
        secrets::encrypt_secret(&state.secrets_key, &plaintext).map_err(AppError::Internal)?;
    let secret = db::update_secret(&conn, &id, &name, &encrypted, &scope)?;
    webhook::dispatch_event(
        &state,
        secrets::EVENT_SECRET_UPDATE,
        secrets::webhook_secret_data(&secret),
    );
    Ok(Json(secret))
}

pub async fn delete_secret(
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
    let existing = db::get_secret(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    db::delete_secret(&conn, &id)?;
    webhook::dispatch_event(
        &state,
        secrets::EVENT_SECRET_DELETE,
        secrets::webhook_secret_data(&existing.unwrap()),
    );
    Ok(StatusCode::NO_CONTENT)
}

// --- WebSocket Handlers ---
