use super::*;

#[tokio::test]
async fn test_notification_auth_required_for_create() {
    let app = test_app().await;
    let body = serde_json::json!({"title": "X", "body": "Y", "type": "info"});
    let (status, _) = post_json(&app, "/api/notifications", body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_collection_create_missing_name() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;
    let body = serde_json::json!({"name": ""});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_collection_create_duplicate() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;
    let body = serde_json::json!({"name": "dup"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let body = serde_json::json!({"name": "dup"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_bucket_create_missing_name() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"name": ""});
    let (status, _) = request(&app, "POST", "/api/buckets", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_record_create_duplicate_id() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;
    let body = serde_json::json!({"name": "items"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"id": "myid", "data": {"x": 1}});
    let (status, _) = request(
        &app,
        "POST",
        "/api/collections/items/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"id": "myid", "data": {"x": 2}});
    let (status, _) = request(
        &app,
        "POST",
        "/api/collections/items/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_record_update_not_found() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;
    let body = serde_json::json!({"name": "items"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"data": {"x": 1}});
    let (status, _) = request(
        &app,
        "PATCH",
        "/api/collections/items/records/nonexistent",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_record_update_missing_data() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;
    let body = serde_json::json!({"name": "items"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"data": {"x": 1}});
    let (status, value) = request(
        &app,
        "POST",
        "/api/collections/items/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = value["id"].as_str().unwrap_or("").to_string();

    let body = serde_json::json!({});
    let (status, _) = request(
        &app,
        "PATCH",
        &format!("/api/collections/items/records/{id}"),
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_pubsub_topic_not_found() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let (status, _) = get_json(&app, "/api/pubsub/topics/nonexistent", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_permissions_crud() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    let body = serde_json::json!({"user_id": "user1", "collection": "*", "role": "viewer"});
    let (status, value) = request(&app, "POST", "/api/permissions", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    let perm_id = value["id"].as_str().unwrap_or("").to_string();

    let (status, value) = get_json(&app, "/api/permissions", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let perms = value.as_array().unwrap();
    assert!(!perms.is_empty());

    let (status, _) = request(
        &app,
        "DELETE",
        &format!("/api/permissions/{perm_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_import_export() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    let body = serde_json::json!({"name": "export_test"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"data": {"val": 1}});
    let (status, _) = request(
        &app,
        "POST",
        "/api/collections/export_test/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, export_data) = get_json(&app, "/api/export", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(export_data.get("collections").is_some());

    let (status, _) = request(
        &app,
        "POST",
        "/api/import?skip_existing=true",
        Some(&token),
        Some(export_data),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}
