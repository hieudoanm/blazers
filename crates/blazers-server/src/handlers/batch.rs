//! Batch requests — PocketBase parity on two surfaces:
//! - `POST /api/batch` with `{"requests": [...]}`
//! - `?batch=1` on any `/api/*` call whose body is a JSON array of
//!   `{method, url, body}` items (or the `{"requests": [...]}` envelope).
//!
//! Each sub-request is dispatched through the real router (auth, RBAC,
//! middleware all apply) and `{status, body}` results return in order.

use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use tower::ServiceExt;

use super::*;

const MAX_BATCH: usize = 50;
const MAX_BATCH_BODY: usize = 10 * 1024 * 1024;
/// Marks sub-requests so the `?batch=1` middleware cannot recurse.
const SUBREQUEST_HEADER: &str = "x-blazers-batch-sub";

fn parse_items(value: &Value) -> Option<Vec<BatchItem>> {
    let arr = if value.is_array() {
        value.as_array().cloned()
    } else {
        value.get("requests")?.as_array().cloned()
    }?;
    serde_json::from_value(Value::Array(arr)).ok()
}

/// Dispatches batch items through a fresh router and collects results.
pub(crate) async fn dispatch_batch(
    state: &std::sync::Arc<AppState>,
    auth_header: Option<&str>,
    items: &[BatchItem],
) -> Vec<Value> {
    // Build the router once per batch call — cheap route-table construction,
    // guarantees sub-requests get identical middleware/auth behavior.
    let router = crate::app::app(state.clone());

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let method = item.method.trim().to_uppercase();
        let url = item.url.trim();
        if !matches!(
            method.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
        ) {
            out.push(serde_json::json!({
                "status": 400,
                "body": {"error": format!("unsupported method '{method}'")},
            }));
            continue;
        }
        if !url.starts_with("/api/") || url.starts_with("/api/batch") {
            out.push(serde_json::json!({
                "status": 400,
                "body": {"error": "batch sub-requests must target /api/* (except /api/batch)"},
            }));
            continue;
        }

        let mut builder = Request::builder()
            .method(method.as_str())
            .uri(format!("http://localhost{url}"))
            .header("content-type", "application/json")
            .header(SUBREQUEST_HEADER, "1");
        if let Some(auth) = auth_header {
            builder = builder.header("authorization", auth);
        }
        let body = if item.body.is_null() {
            Body::empty()
        } else {
            Body::from(serde_json::to_vec(&item.body).unwrap_or_default())
        };
        let request = match builder.body(body) {
            Ok(r) => r,
            Err(e) => {
                out.push(serde_json::json!({
                    "status": 400,
                    "body": {"error": format!("bad request: {e}")},
                }));
                continue;
            }
        };

        match router.clone().oneshot(request).await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap_or_default();
                let body: Value = serde_json::from_slice(&bytes)
                    .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).to_string()));
                out.push(serde_json::json!({ "status": status, "body": body }));
            }
            Err(e) => {
                out.push(serde_json::json!({
                    "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "body": {"error": format!("dispatch failed: {e}")},
                }));
            }
        }
    }
    out
}

fn batch_auth_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

fn validate_items(items: Vec<BatchItem>) -> std::result::Result<Vec<BatchItem>, AppError> {
    if items.is_empty() {
        return Err(AppError::BadRequest("requests must not be empty".into()));
    }
    if items.len() > MAX_BATCH {
        return Err(AppError::BadRequest(format!(
            "batch too large (max {MAX_BATCH})"
        )));
    }
    Ok(items)
}

/// POST /api/batch — `{"requests": [{method, url, body}, ...]}`.
pub async fn handle_batch(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<BatchRequest>,
) -> std::result::Result<Json<Vec<Value>>, AppError> {
    extract_claims(&headers)?;
    let items = validate_items(req.requests)?;
    let out = dispatch_batch(&state, batch_auth_header(&headers).as_deref(), &items).await;
    Ok(Json(out))
}

fn wants_batch(req: &Request) -> bool {
    if req.headers().contains_key(SUBREQUEST_HEADER) {
        return false;
    }
    let path = req.uri().path();
    if !path.starts_with("/api/") || path == "/api/batch" {
        return false;
    }
    req.uri()
        .query()
        .map(|q| {
            q.split('&')
                .any(|pair| pair == "batch=1" || pair == "batch=true" || pair == "batch")
        })
        .unwrap_or(false)
}

/// `?batch=1` middleware — any `/api/*` request (except `/api/batch` and
/// batch sub-requests) whose body parses as a batch payload is answered
/// with the array of per-request results.
pub async fn batch_query_middleware(
    State(state): State<std::sync::Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    if !wants_batch(&req) {
        return next.run(req).await;
    }
    let auth = batch_auth_header(req.headers());
    let (_, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, MAX_BATCH_BODY).await {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({"error": "batch body too large"})),
            )
                .into_response();
        }
    };
    let value: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "batch body must be a JSON array or {\"requests\": [...] }"})),
            )
                .into_response();
        }
    };
    let Some(items) = parse_items(&value) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "batch body must be a JSON array or {\"requests\": [...] }"})),
        )
            .into_response();
    };
    let items = match validate_items(items) {
        Ok(i) => i,
        Err(e) => return e.into_response(),
    };
    let out = dispatch_batch(&state, auth.as_deref(), &items).await;
    Json(out).into_response()
}
