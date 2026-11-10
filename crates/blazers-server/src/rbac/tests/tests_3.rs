use super::*;

#[tokio::test]
async fn test_handle_delete_permission_not_found() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let (status, value) = request_with_auth(
        &app,
        axum::http::Method::DELETE,
        "/api/permissions/nonexistent-id",
        &token,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    assert_eq!(value["error"], "permission not found");
}

#[tokio::test]
async fn test_handle_create_permission_missing_fields() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let body = serde_json::json!({"user_id": "", "collection": "docs", "role": "viewer"});
    let (status, value) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert!(value["error"].as_str().unwrap_or("").contains("required"));
}

#[tokio::test]
async fn test_handle_create_permission_invalid_role() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "superadmin"});
    let (status, value) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert!(
        value["error"]
            .as_str()
            .unwrap_or("")
            .contains("role must be")
    );
}

#[tokio::test]
async fn test_handle_permissions_requires_auth() {
    let (app, _state) = test_rbac_app().await;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let req = Request::builder()
        .uri("/api/permissions")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

    let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
    let req = Request::builder()
        .method("POST")
        .uri("/api/permissions")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

    let req = Request::builder()
        .method("DELETE")
        .uri("/api/permissions/some-id")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_handle_create_permission_duplicate() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
    let (status, _) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body.clone()),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let (status, _) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}
