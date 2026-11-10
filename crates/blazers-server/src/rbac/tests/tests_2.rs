use super::*;

#[test]
fn test_find_permission_by_id_not_found() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    assert!(
        find_permission_by_id(&conn, "nonexistent")
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn test_handle_list_permissions_empty() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let (status, value) = request_with_auth(
        &app,
        axum::http::Method::GET,
        "/api/permissions",
        &token,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(value.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_handle_create_permission() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
    let (status, value) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert!(!value["id"].as_str().unwrap_or("").is_empty());
    assert_eq!(value["user_id"], "user1");
    assert_eq!(value["collection"], "docs");
    assert_eq!(value["role"], "viewer");
}

#[tokio::test]
async fn test_handle_list_permissions_after_create() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
    let (status, _) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let (status, value) = request_with_auth(
        &app,
        axum::http::Method::GET,
        "/api/permissions",
        &token,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let perms = value.as_array().unwrap();
    assert_eq!(perms.len(), 1);
    assert_eq!(perms[0]["user_id"], "user1");
}

#[tokio::test]
async fn test_handle_delete_permission() {
    let (app, _state) = test_rbac_app().await;
    let token = test_token();
    let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
    let (status, value) = request_with_auth(
        &app,
        axum::http::Method::POST,
        "/api/permissions",
        &token,
        Some(body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let perm_id = value["id"].as_str().unwrap().to_string();
    let (status, _) = request_with_auth(
        &app,
        axum::http::Method::DELETE,
        &format!("/api/permissions/{perm_id}"),
        &token,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
}
