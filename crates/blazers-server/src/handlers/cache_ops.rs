use super::*;

#[derive(serde::Deserialize)]
pub struct ListRangeQuery {
    pub start: Option<i64>,
    pub stop: Option<i64>,
}

pub async fn list_cache_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
    axum::extract::Query(q): axum::extract::Query<ListRangeQuery>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let items = state
        .cache
        .list_range(&conn, &key, q.start.unwrap_or(0), q.stop.unwrap_or(-1))?;
    Ok(Json(serde_json::json!({"key": key, "items": items})))
}

pub async fn push_cache_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<blazers_core::models::CacheListPushRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let len = state.cache.list_push(&conn, &key, req.values, req.left)?;
    Ok(Json(serde_json::json!({"key": key, "length": len})))
}

pub async fn pop_cache_list(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<blazers_core::models::CacheListPopRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let value = state.cache.list_pop(&conn, &key, req.left, req.count)?;
    Ok(Json(serde_json::json!({"key": key, "value": value})))
}

pub async fn get_cache_hash(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let fields = state.cache.hash_get_all(&conn, &key)?;
    Ok(Json(serde_json::json!({"key": key, "fields": fields})))
}

pub async fn get_cache_hash_field(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((key, field)): Path<(String, String)>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let value = state.cache.hash_get(&conn, &key, &field)?;
    Ok(Json(
        serde_json::json!({"key": key, "field": field, "value": value}),
    ))
}

pub async fn set_cache_hash(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<blazers_core::models::CacheHashSetRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut fields = req.fields.unwrap_or_default();
    if let (Some(field), Some(value)) = (req.field, req.value) {
        fields.insert(field, value);
    }
    if fields.is_empty() {
        return Err(AppError::BadRequest(
            "field/value or fields is required".into(),
        ));
    }
    let n = state.cache.hash_set(&conn, &key, fields)?;
    Ok(Json(serde_json::json!({"key": key, "fields_set": n})))
}

pub async fn delete_cache_hash_fields(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path((key, fields)): Path<(String, String)>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let names: Vec<String> = fields.split(',').map(|s| s.trim().to_string()).collect();
    let removed = state.cache.hash_del(&conn, &key, &names)?;
    Ok(Json(serde_json::json!({"key": key, "removed": removed})))
}

pub async fn delete_cache_keys(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<blazers_core::models::DeleteCacheKeysRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    if req.keys.is_empty() {
        return Err(AppError::BadRequest("keys is required".into()));
    }
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let removed = state.cache.delete_many(&conn, &req.keys)?;
    Ok(Json(serde_json::json!({"deleted": removed})))
}

pub async fn cache_key_exists(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let exists = state.cache.exists(&conn, &key)?;
    Ok(Json(serde_json::json!({"key": key, "exists": exists})))
}

pub async fn list_cache_keys(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    Ok(Json(serde_json::json!({"keys": state.cache.keys()})))
}
