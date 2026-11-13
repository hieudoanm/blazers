//! Protected-file access: public buckets, `?token=` downloads, token issuing.

use super::*;

#[derive(serde::Deserialize)]
pub struct FileAccessQuery {
    pub token: Option<String>,
}

/// True when the request may access the file: public bucket, valid file
/// token, or an authenticated session.
pub(crate) async fn file_access_allowed(
    state: &std::sync::Arc<AppState>,
    headers: &HeaderMap,
    bucket_name: &str,
    file_id: &str,
    token: Option<&str>,
) -> std::result::Result<bool, AppError> {
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(b) = db::get_bucket(&conn, bucket_name)?
        && b.is_public
    {
        return Ok(true);
    }
    if let Some(t) = token
        && auth::validate_file_token(t).ok().as_deref() == Some(file_id)
    {
        return Ok(true);
    }
    drop(conn);
    Ok(extract_claims(headers).is_ok())
}

/// POST /api/buckets/{bucket}/files/{file_id}/token — issues a short-lived
/// download token (default 1h) for protected files. An optional JSON body
/// `{collection, record}` binds the token to that record's `view_rule`:
/// the caller must pass the rule for the token to be issued.
pub async fn create_file_token(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((bucket, file_id)): Path<(String, String)>,
    body: Option<Json<FileTokenRequest>>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let f = db::get_file(&conn, &file_id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    if f.bucket != bucket {
        return Err(AppError::NotFound("not found".into()));
    }

    if let Some(Json(req)) = body
        && !req.collection.is_empty()
        && !req.record.is_empty()
    {
        let collection = db::get_collection(&conn, &req.collection)?
            .ok_or_else(|| AppError::NotFound("collection not found".into()))?;
        let record = db::get_record(&conn, &req.collection, &req.record)?
            .ok_or_else(|| AppError::NotFound("record not found".into()))?;
        let auth_ctx = rbac::auth_ctx(&conn, &claims.user_id, &claims.email);
        rbac::check_record_rule(
            &conn,
            &collection,
            rbac::RuleField::View,
            &record,
            &auth_ctx,
            &claims.user_id,
        )?;
    }

    Ok(Json(serde_json::json!({
        "token": auth::generate_file_token(&file_id, 3600),
        "expires_in": 3600,
    })))
}
