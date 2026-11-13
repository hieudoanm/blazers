use super::*;
use crate::auth::Claims;

fn record_auth_ctx(conn: &rusqlite::Connection, claims: &Claims) -> serde_json::Value {
    rbac::auth_ctx(conn, &claims.user_id, &claims.email)
}

pub async fn create_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Json(mut req): Json<CreateRecordRequest>,
) -> std::result::Result<Json<Record>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    rbac::require_role(&conn, &claims.user_id, &name, "editor")?;
    let collection = db::get_collection(&conn, &name)?
        .ok_or_else(|| AppError::NotFound("collection not found".into()))?;
    db::validate_data(&req.data, &collection.schema)?;
    let auth = record_auth_ctx(&conn, &claims);
    // create rule evaluates against the *incoming* record payload
    let incoming = db::record_view(&Record {
        id: req.id.clone().unwrap_or_default(),
        data: req.data.clone(),
        created_at: String::new(),
        updated_at: String::new(),
    });
    rbac::check_rule_view(
        &conn,
        &collection,
        &collection.create_rule,
        &incoming,
        &auth,
        &claims.user_id,
    )?;
    // Synchronous pre-write hook: may patch `data` or cancel.
    match webhook::run_record_hooks(
        &state,
        webhook::before_event("create"),
        &name,
        &incoming,
        &auth,
    )
    .await?
    {
        webhook::HookOutcome::Cancel => {
            return Err(AppError::BadRequest(
                "record creation cancelled by hook".into(),
            ));
        }
        webhook::HookOutcome::Continue(Some(patch)) => {
            webhook::merge_data_patch(&mut req.data, patch);
            db::validate_data(&req.data, &collection.schema)?;
        }
        _ => {}
    }
    let id = req
        .id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string().replace('-', ""));
    let rec = db::insert_record(&conn, &name, &id, &req.data)?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_CREATE,
        webhook::webhook_record_data(&name, &rec),
    );
    websocket::broadcast_record(
        &state.ws_hub,
        &name,
        "create",
        &serde_json::to_value(&rec).unwrap_or(Value::Null),
    )
    .await;
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
    let collection = db::get_collection(&conn, &name)?
        .ok_or_else(|| AppError::NotFound("collection not found".into()))?;

    let auth = record_auth_ctx(&conn, &claims);
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let search = params.search.as_deref().unwrap_or("");

    // Combine the client `filter` with the collection's list rule so both
    // apply before pagination. Admins bypass the rule.
    let rule = collection.list_rule.trim();
    let effective_filter = if rule.is_empty() || rbac::is_admin(&conn, &claims.user_id, &name)? {
        params.filter.clone()
    } else {
        match params
            .filter
            .as_deref()
            .map(str::trim)
            .filter(|f| !f.is_empty())
        {
            Some(f) => Some(format!("({f}) && ({rule})")),
            None => Some(rule.to_string()),
        }
    };

    let records = db::list_records_query(
        &conn,
        &name,
        page,
        per_page,
        search,
        effective_filter.as_deref(),
        params.sort.as_deref(),
        &auth,
    )?;

    let mut records = records;
    if let Some(expand) = params.expand.as_deref().filter(|e| !e.trim().is_empty()) {
        expand_records(&conn, &mut records.records, expand)?;
    }
    Ok(Json(records))
}
