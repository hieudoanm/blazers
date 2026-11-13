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
    let mut cols = db::list_collections(&conn)?;
    // The managed `users` auth collection is synthetic — backed by `_users`,
    // not a dynamic table.
    if !cols.iter().any(|c| c.name == "users") {
        cols.insert(0, users_collection());
    }
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
    db::validate_schema(&req.schema)?;
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
    if name == "users" {
        return Ok(Json(users_collection()));
    }
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
    if let Some(schema) = req.schema.as_deref() {
        db::validate_schema(schema)?;
    }
    let c = db::update_collection(&conn, &name, req.schema.as_deref())?;
    if req.list_rule.is_some()
        || req.view_rule.is_some()
        || req.create_rule.is_some()
        || req.update_rule.is_some()
        || req.delete_rule.is_some()
    {
        // Merge provided rules over the current values.
        let mut rules = CollectionRules {
            list_rule: c.list_rule.clone(),
            view_rule: c.view_rule.clone(),
            create_rule: c.create_rule.clone(),
            update_rule: c.update_rule.clone(),
            delete_rule: c.delete_rule.clone(),
        };
        if let Some(v) = req.list_rule {
            rules.list_rule = v;
        }
        if let Some(v) = req.view_rule {
            rules.view_rule = v;
        }
        if let Some(v) = req.create_rule {
            rules.create_rule = v;
        }
        if let Some(v) = req.update_rule {
            rules.update_rule = v;
        }
        if let Some(v) = req.delete_rule {
            rules.delete_rule = v;
        }
        db::update_collection_rules(&conn, &name, &rules)?;
        return Ok(Json(db::get_collection(&conn, &name)?.unwrap_or(c)));
    }
    Ok(Json(c))
}

/// Synthetic collection descriptor for the managed auth `users` collection.
fn users_collection() -> Collection {
    Collection {
        name: "users".into(),
        schema: "{}".into(),
        list_rule: String::new(),
        view_rule: String::new(),
        create_rule: String::new(),
        update_rule: String::new(),
        delete_rule: String::new(),
        r#type: "auth".into(),
        created_at: String::new(),
        updated_at: String::new(),
    }
}
