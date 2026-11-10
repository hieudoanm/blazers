use super::*;

pub async fn create_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<CreateRecordRequest>,
) -> std::result::Result<Json<Record>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "editor")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("collection not found".into()));
    }
    let collection = existing.unwrap();
    db::validate_data(&req.data, &collection.schema)?;
    let id = req
        .id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string().replace('-', ""));
    let rec = db::insert_record(&conn, &name, &id, &req.data)?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_CREATE,
        webhook::webhook_record_data(&name, &rec),
    );
    Ok(Json(rec))
}

pub async fn list_records(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<RecordListParams>,
) -> std::result::Result<Json<RecordsPage>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "viewer")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("collection not found".into()));
    }
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let search = params.search.as_deref().unwrap_or("");
    let records = db::list_records(&conn, &name, page, per_page, search)?;
    Ok(Json(records))
}

pub async fn get_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((name, id)): Path<(String, String)>,
) -> std::result::Result<Json<Record>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "viewer")?;
    let rec =
        db::get_record(&conn, &name, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(rec))
}

pub async fn update_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((name, id)): Path<(String, String)>,
    Json(req): Json<UpdateRecordRequest>,
) -> std::result::Result<Json<Record>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "editor")?;
    let existing = db::get_collection(&conn, &name)?;
    if existing.is_none() {
        return Err(AppError::NotFound("collection not found".into()));
    }
    let collection = existing.unwrap();
    db::validate_data(&req.data, &collection.schema)?;
    let rec = db::update_record(&conn, &name, &id, &req.data)?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_UPDATE,
        webhook::webhook_record_data(&name, &rec),
    );
    Ok(Json(rec))
}

pub async fn delete_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((name, id)): Path<(String, String)>,
) -> std::result::Result<StatusCode, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "admin")?;
    let existing = db::get_record(&conn, &name, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let rec = existing.unwrap();
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_DELETE,
        webhook::webhook_record_data(&name, &rec),
    );
    db::delete_record(&conn, &name, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
