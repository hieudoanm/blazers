use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

fn test_token() -> String {
    use chrono::Utc;
    use jsonwebtoken::{EncodingKey, Header, encode};
    let claims = auth::Claims {
        user_id: "test-user-id".into(),
        email: "test@example.com".into(),
        exp: Utc::now().timestamp() as usize + 3600,
        iat: Utc::now().timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".into())
                .as_bytes(),
        ),
    )
    .unwrap()
}

// --- extract_token ---

#[test]
fn test_extract_token_valid() {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "Bearer my-token".parse().unwrap());
    assert_eq!(extract_token(&headers).unwrap(), "my-token");
}

#[test]
fn test_extract_token_missing_header() {
    let headers = HeaderMap::new();
    match extract_token(&headers).unwrap_err() {
        AppError::Unauthorized(_) => {}
        other => panic!("expected Unauthorized, got {other:?}"),
    }
}

#[test]
fn test_extract_token_missing_bearer_prefix() {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "Basic xxx".parse().unwrap());
    match extract_token(&headers).unwrap_err() {
        AppError::Unauthorized(_) => {}
        other => panic!("expected Unauthorized, got {other:?}"),
    }
}

#[test]
fn test_extract_token_invalid_utf8() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        axum::http::HeaderValue::from_bytes(b"Bearer \xff\xff").unwrap(),
    );
    match extract_token(&headers).unwrap_err() {
        AppError::Unauthorized(_) => {}
        other => panic!("expected Unauthorized, got {other:?}"),
    }
}

// --- extract_claims ---

#[test]
fn test_extract_claims_valid() {
    let token = test_token();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {token}").parse().unwrap());
    let claims = extract_claims(&headers).unwrap();
    assert_eq!(claims.user_id, "test-user-id");
    assert_eq!(claims.email, "test@example.com");
}

#[test]
fn test_extract_claims_invalid_token() {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", "Bearer garbage-token".parse().unwrap());
    match extract_claims(&headers).unwrap_err() {
        AppError::Unauthorized(_) => {}
        other => panic!("expected Unauthorized, got {other:?}"),
    }
}

#[test]
fn test_extract_claims_missing_header() {
    let headers = HeaderMap::new();
    match extract_claims(&headers).unwrap_err() {
        AppError::Unauthorized(_) => {}
        other => panic!("expected Unauthorized, got {other:?}"),
    }
}

// --- get_openapi_json ---

#[tokio::test]
async fn test_get_openapi_json_returns_json() {
    let app = axum::Router::new().route("/api/openapi.json", axum::routing::get(get_openapi_json));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("application/json"),
        "expected json content type, got {ct}"
    );
}

// --- get_swagger_ui ---

#[tokio::test]
async fn test_get_swagger_ui_returns_html() {
    let app = axum::Router::new().route("/api/docs", axum::routing::get(get_swagger_ui));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("text/html"),
        "expected html content type, got {ct}"
    );
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("SwaggerUIBundle"),
        "expected SwaggerUIBundle in body"
    );
}

// --- get_health ---

#[tokio::test]
async fn test_get_health_returns_ok() {
    let app = axum::Router::new().route("/api/health", axum::routing::get(get_health));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(val["status"], "ok");
}
