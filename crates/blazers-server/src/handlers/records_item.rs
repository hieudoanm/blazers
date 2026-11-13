//! Single-record handlers (view/update/delete with per-record rules).

use super::*;
use crate::auth::Claims;

fn record_auth_ctx(conn: &rusqlite::Connection, claims: &Claims) -> serde_json::Value {
    rbac::auth_ctx(conn, &claims.user_id, &claims.email)
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
    let collection = db::get_collection(&conn, &name)?
        .ok_or_else(|| AppError::NotFound("collection not found".into()))?;
    let rec =
        db::get_record(&conn, &name, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    let auth = record_auth_ctx(&conn, &claims);
    rbac::check_record_rule(
        &conn,
        &collection,
        rbac::RuleField::View,
        &rec,
        &auth,
        &claims.user_id,
    )?;
    Ok(Json(rec))
}

pub async fn update_record(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((name, id)): Path<(String, String)>,
    Json(mut req): Json<UpdateRecordRequest>,
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
    let existing =
        db::get_record(&conn, &name, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    let auth = record_auth_ctx(&conn, &claims);
    rbac::check_record_rule(
        &conn,
        &collection,
        rbac::RuleField::Update,
        &existing,
        &auth,
        &claims.user_id,
    )?;
    // Also evaluate the create rule against the merged result so rules like
    // "status must be pending on insert" still apply to updates.
    let mut merged = existing.data.clone();
    if let (Value::Object(m), Value::Object(new)) = (&mut merged, &req.data) {
        for (k, v) in new {
            m.insert(k.clone(), v.clone());
        }
    } else {
        merged = req.data.clone();
    }
    let merged_view = db::record_view(&Record {
        id: existing.id.clone(),
        data: merged,
        created_at: existing.created_at.clone(),
        updated_at: existing.updated_at.clone(),
    });
    rbac::check_rule_view(
        &conn,
        &collection,
        &collection.create_rule,
        &merged_view,
        &auth,
        &claims.user_id,
    )?;
    // Synchronous pre-write hook: may patch `data` or cancel.
    match webhook::run_record_hooks(
        &state,
        webhook::before_event("update"),
        &name,
        &merged_view,
        &auth,
    )
    .await?
    {
        webhook::HookOutcome::Cancel => {
            return Err(AppError::BadRequest(
                "record update cancelled by hook".into(),
            ));
        }
        webhook::HookOutcome::Continue(Some(patch)) => {
            webhook::merge_data_patch(&mut req.data, patch);
            db::validate_data(&req.data, &collection.schema)?;
        }
        _ => {}
    }
    let rec = db::update_record(&conn, &name, &id, &req.data)?;
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_UPDATE,
        webhook::webhook_record_data(&name, &rec),
    );
    websocket::broadcast_record_change(
        &state.ws_hub,
        &name,
        "update",
        &serde_json::to_value(&rec).unwrap_or(Value::Null),
        Some(&db::record_view(&existing)),
    )
    .await;
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
    rbac::require_role(&conn, &claims.user_id, &name, "editor")?;
    let collection = db::get_collection(&conn, &name)?
        .ok_or_else(|| AppError::NotFound("collection not found".into()))?;
    let rec =
        db::get_record(&conn, &name, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    let auth = record_auth_ctx(&conn, &claims);
    rbac::check_record_rule(
        &conn,
        &collection,
        rbac::RuleField::Delete,
        &rec,
        &auth,
        &claims.user_id,
    )?;
    // Synchronous pre-delete hook: may cancel the operation.
    let view = db::record_view(&rec);
    if let webhook::HookOutcome::Cancel =
        webhook::run_record_hooks(&state, webhook::before_event("delete"), &name, &view, &auth)
            .await?
    {
        return Err(AppError::BadRequest(
            "record deletion cancelled by hook".into(),
        ));
    }
    webhook::dispatch_event(
        &state,
        webhook::EVENT_RECORD_DELETE,
        webhook::webhook_record_data(&name, &rec),
    );
    db::delete_record(&conn, &name, &id)?;
    websocket::broadcast_record_change(
        &state.ws_hub,
        &name,
        "delete",
        &serde_json::to_value(&rec).unwrap_or(Value::Null),
        Some(&view),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}
