//! HTTP handlers for `/api/permissions`.

use super::*;

pub async fn handle_list_permissions(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Permission>>, AppError> {
    crate::handlers::extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut stmt = conn
        .prepare("SELECT id, user_id, collection, role, created_at, updated_at FROM _permissions ORDER BY collection")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Permission {
                id: row.get(0)?,
                user_id: row.get(1)?,
                collection: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut perms = Vec::new();
    for row in rows {
        perms.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(Json(perms))
}

pub async fn handle_create_permission(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<PermissionRequest>,
) -> std::result::Result<(StatusCode, Json<Permission>), AppError> {
    crate::handlers::extract_claims(&headers)?;
    if req.user_id.is_empty() || req.collection.is_empty() || req.role.is_empty() {
        return Err(AppError::BadRequest(
            "user_id, collection, and role are required".into(),
        ));
    }
    if !matches!(req.role.as_str(), "admin" | "editor" | "viewer") {
        return Err(AppError::BadRequest(
            "role must be admin, editor, or viewer".into(),
        ));
    }
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute(
        "INSERT INTO _permissions (id, user_id, collection, role) VALUES (?1, ?2, ?3, ?4)",
        params![id, req.user_id, req.collection, req.role],
    )
    .map_err(|e| AppError::Internal(format!("insert permission: {e}")))?;
    let perm = find_permission_by_id(&conn, &id)?
        .ok_or_else(|| AppError::Internal("permission not found after create".into()))?;
    Ok((StatusCode::CREATED, Json(perm)))
}

pub async fn handle_delete_permission(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    crate::handlers::extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n = conn
        .execute("DELETE FROM _permissions WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound("permission not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn find_permission_by_id(
    conn: &Connection,
    id: &str,
) -> std::result::Result<Option<Permission>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, user_id, collection, role, created_at, updated_at FROM _permissions WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(Permission {
                id: row.get(0)?,
                user_id: row.get(1)?,
                collection: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(p)) => Ok(Some(p)),
        _ => Ok(None),
    }
}
