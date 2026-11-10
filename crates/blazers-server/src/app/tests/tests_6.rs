use super::*;

#[tokio::test]
async fn test_notification_mark_read() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"title": "Read Me", "type": "info"});
    let (status, value) =
        request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let id = value["id"].as_str().unwrap().to_string();

    let (status, value) = request(
        &app,
        "PATCH",
        &format!("/api/notifications/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["is_read"], true);

    let (status, value) = get_json(&app, &format!("/api/notifications/{id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["is_read"], true);
}

#[tokio::test]
async fn test_notification_delete() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"title": "To Delete", "type": "info"});
    let (status, value) =
        request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let id = value["id"].as_str().unwrap().to_string();

    let (status, _) = request(
        &app,
        "DELETE",
        &format!("/api/notifications/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = get_json(&app, &format!("/api/notifications/{id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_notification_validation() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"title": "", "type": "info"});
    let (status, _) = request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let body = serde_json::json!({"title": "Bad Type", "type": "invalid"});
    let (status, _) = request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_notification_clear() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"title": "N1", "type": "info"});
    let (status, _) = request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"title": "N2", "type": "info"});
    let (status, _) = request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = request(&app, "DELETE", "/api/notifications", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, value) = get_json(&app, "/api/notifications", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_notification_auth_required() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/notifications", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = post_json(
        &app,
        "/api/notifications",
        serde_json::json!({"title": "x", "type": "info"}),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_log_create() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body =
        serde_json::json!({"level": "info", "message": "test log", "meta": "{\"key\":\"val\"}"});
    let (status, value) = request(&app, "POST", "/api/logs", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["message"], "test log");
    assert_eq!(value["level"], "info");
    assert!(!value["id"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn test_log_list() {
    let app = test_app().await;
    let token = register_token(&app).await;

    for i in 0..3 {
        let body = serde_json::json!({"level": "info", "message": format!("log {i}")});
        let (status, _) = request(&app, "POST", "/api/logs", Some(&token), Some(body)).await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, value) = get_json(&app, "/api/logs", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[tokio::test]
async fn test_log_validation() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"level": "info", "message": ""});
    let (status, _) = request(&app, "POST", "/api/logs", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let body = serde_json::json!({"level": "invalid", "message": "test"});
    let (status, _) = request(&app, "POST", "/api/logs", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_log_clear() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"level": "info", "message": "test"});
    let (status, _) = request(&app, "POST", "/api/logs", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = request(&app, "DELETE", "/api/logs", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, value) = get_json(&app, "/api/logs", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_log_auth_required() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/logs", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = post_json(
        &app,
        "/api/logs",
        serde_json::json!({"message": "x", "level": "info"}),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
