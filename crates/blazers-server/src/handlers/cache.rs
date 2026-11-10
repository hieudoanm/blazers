use super::*;

pub async fn list_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<CacheEntry>>, AppError> {
    extract_claims(&headers)?;
    Ok(Json(state.cache.list()))
}

pub async fn set_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<SetCacheRequest>,
) -> std::result::Result<Json<CacheEntry>, AppError> {
    extract_claims(&headers)?;
    if req.key.is_empty() || req.value.is_empty() {
        return Err(AppError::BadRequest("key and value are required".into()));
    }
    let ttl = req.ttl.unwrap_or(0);
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let entry = state.cache.set(&conn, &req.key, &req.value, ttl)?;
    Ok(Json(entry))
}

pub async fn get_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
) -> std::result::Result<Json<CacheEntry>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let entry = state
        .cache
        .get(&conn, &key)?
        .ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(entry))
}

pub async fn delete_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let ok = state.cache.delete(&conn, &key)?;
    if !ok {
        return Err(AppError::NotFound("not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn flush_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    state.cache.flush(&conn)?;
    Ok(Json(serde_json::json!({"status": "flushed"})))
}

pub async fn get_cache_stats(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    Ok(Json(state.cache.stats()))
}

pub async fn incr_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<blazers_core::models::CacheIncrRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let value = state.cache.incr(&conn, &key, req.by)?;
    Ok(Json(serde_json::json!({"key": key, "value": value})))
}

pub async fn append_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<blazers_core::models::CacheAppendRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let value = state.cache.append(&conn, &key, &req.value)?;
    Ok(Json(serde_json::json!({"key": key, "value": value})))
}

pub async fn expire_cache(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<blazers_core::models::CacheExpireRequest>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match state.cache.expire(&conn, &key, req.ttl)? {
        Some(()) => Ok(Json(serde_json::json!({"key": key, "ttl": req.ttl}))),
        None => Err(AppError::NotFound("not found".into())),
    }
}

pub async fn get_cache_ttl(
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
    match state.cache.ttl(&conn, &key)? {
        Some(ttl) => Ok(Json(serde_json::json!({"key": key, "ttl": ttl}))),
        None => Err(AppError::NotFound("not found".into())),
    }
}
