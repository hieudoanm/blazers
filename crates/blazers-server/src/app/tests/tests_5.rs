use super::*;

#[tokio::test]
async fn test_secrets_auth_required() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/secrets", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// --- Cache Tests ---

#[tokio::test]
async fn test_cache_set_get_delete() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let (status, val) = request(
        &app,
        "POST",
        "/api/cache",
        Some(&token),
        Some(serde_json::json!({"key": "hello", "value": "world"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(val["key"], "hello");
    assert_eq!(val["value"], "world");

    let (status, val) = get_json(&app, "/api/cache/hello", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(val["value"], "world");

    let (status, _) = request(&app, "DELETE", "/api/cache/hello", Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = get_json(&app, "/api/cache/hello", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cache_list() {
    let app = test_app().await;
    let token = register_token(&app).await;

    request(
        &app,
        "POST",
        "/api/cache",
        Some(&token),
        Some(serde_json::json!({"key": "a", "value": "1"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/api/cache",
        Some(&token),
        Some(serde_json::json!({"key": "b", "value": "2"})),
    )
    .await;

    let (status, val) = get_json(&app, "/api/cache", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(val.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_cache_overwrite() {
    let app = test_app().await;
    let token = register_token(&app).await;

    request(
        &app,
        "POST",
        "/api/cache",
        Some(&token),
        Some(serde_json::json!({"key": "k", "value": "v1"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/api/cache",
        Some(&token),
        Some(serde_json::json!({"key": "k", "value": "v2"})),
    )
    .await;

    let (_, val) = get_json(&app, "/api/cache/k", Some(&token)).await;
    assert_eq!(val["value"], "v2");
}

#[tokio::test]
async fn test_cache_flush() {
    let app = test_app().await;
    let token = register_token(&app).await;

    request(
        &app,
        "POST",
        "/api/cache",
        Some(&token),
        Some(serde_json::json!({"key": "x", "value": "1"})),
    )
    .await;

    let (status, _) = request(&app, "DELETE", "/api/cache", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);

    let (_, val) = get_json(&app, "/api/cache", Some(&token)).await;
    assert_eq!(val.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_cache_validation() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let (status, _) = request(
        &app,
        "POST",
        "/api/cache",
        Some(&token),
        Some(serde_json::json!({"key": "k"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = get_json(&app, "/api/cache/nonexistent", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cache_auth_required() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/cache", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_notification_create() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"title": "Test Notification", "body": "Hello", "type": "info"});
    let (status, value) =
        request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["title"], "Test Notification");
    assert!(!value["id"].as_str().unwrap_or("").is_empty());
    assert_eq!(value["type"], "info");
    assert_eq!(value["is_read"], false);
}

#[tokio::test]
async fn test_notification_list() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"title": "N1", "type": "success"});
    let (status, _) = request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"title": "N2", "type": "warning"});
    let (status, _) = request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, value) = get_json(&app, "/api/notifications", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}
