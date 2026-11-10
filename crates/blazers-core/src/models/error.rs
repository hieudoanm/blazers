use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    /// Prints only the message, so a handler can surface the cause to a caller
    /// without also leaking the HTTP status it maps to.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            AppError::BadRequest(m)
            | AppError::NotFound(m)
            | AppError::Conflict(m)
            | AppError::Unauthorized(m)
            | AppError::Forbidden(m)
            | AppError::Internal(m) => m,
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, m),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        (code, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
