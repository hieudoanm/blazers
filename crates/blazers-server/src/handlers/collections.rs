use super::*;

pub async fn list_collections(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Collection>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let cols = db::list_collections(&conn)?;
    Ok(Json(cols))
}

pub async fn create_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateCollectionRequest>,
) -> std::result::Result<Json<Collection>, AppError> {
    let claims = extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &req.name, "admin")?;
    db::insert_collection(&conn, &req.name, &req.schema)?;
    db::create_collection_table(&conn, &req.name, &req.schema)?;
    let c = db::get_collection(&conn, &req.name)?
        .ok_or_else(|| AppError::Internal("collection not found after create".into()))?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_COLLECTION_CREATE,
        webhook::webhook_collection_data(&c),
    );
    Ok(Json(c))
}

pub async fn get_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Json<Collection>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let c =
        db::get_collection(&conn, &name)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(c))
}

pub async fn delete_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "admin")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let c = existing.unwrap();
    webhook::dispatch_event(
        &state,
        webhook::EVENT_COLLECTION_DELETE,
        webhook::webhook_collection_data(&c),
    );
    db::delete_collection(&conn, &name)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_collection(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<UpdateCollectionRequest>,
) -> std::result::Result<Json<Collection>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "admin")?;
    let c = db::update_collection(&conn, &name, req.schema.as_deref())?;
    Ok(Json(c))
}
