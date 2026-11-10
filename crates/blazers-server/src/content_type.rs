use axum::{
    Json,
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn require_json_content_type(request: Request, next: Next) -> Response {
    if request.method() == Method::POST
        || request.method() == Method::PATCH
        || request.method() == Method::PUT
    {
        let content_type = request
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !content_type.starts_with("application/json")
            && !content_type.starts_with("multipart/form-data")
        {
            return (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Json(serde_json::json!({"error": "content-type must be application/json or multipart/form-data"})),
            ).into_response();
        }
    }
    next.run(request).await
}
