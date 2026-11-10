use super::*;

#[tokio::test]
async fn test_pubsub_topic_crud() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"name": "mytopic"});
    let (status, value) =
        request(&app, "POST", "/api/pubsub/topics", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(value["name"], "mytopic");

    let (status, value) = get_json(&app, "/api/pubsub/topics", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value.as_array().unwrap().len(), 1);

    let (status, _) = request(
        &app,
        "GET",
        "/api/pubsub/topics/mytopic",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = request(
        &app,
        "DELETE",
        "/api/pubsub/topics/mytopic",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(
        &app,
        "GET",
        "/api/pubsub/topics/mytopic",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_pubsub_topic_duplicate() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"name": "dup"});
    request(&app, "POST", "/api/pubsub/topics", Some(&token), Some(body)).await;
    let body = serde_json::json!({"name": "dup"});
    let (status, _) = request(&app, "POST", "/api/pubsub/topics", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_pubsub_topic_missing_name() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({});
    let (status, _) = request(&app, "POST", "/api/pubsub/topics", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_pubsub_topic_auth_required() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/pubsub/topics", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_pubsub_message_publish_list() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"name": "chat"});
    request(&app, "POST", "/api/pubsub/topics", Some(&token), Some(body)).await;

    let body = serde_json::json!({"body": "hello world"});
    let (status, value) = request(
        &app,
        "POST",
        "/api/pubsub/topics/chat/messages",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(value["body"], "hello world");
    assert!(!value["topic_id"].as_str().unwrap_or("").is_empty());

    let (status, value) = request(
        &app,
        "GET",
        "/api/pubsub/topics/chat/messages",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_pubsub_message_to_unknown_topic() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"body": "x"});
    let (status, _) = request(
        &app,
        "POST",
        "/api/pubsub/topics/nonexistent/messages",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_pubsub_message_missing_body() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"name": "t"});
    request(&app, "POST", "/api/pubsub/topics", Some(&token), Some(body)).await;
    let body = serde_json::json!({});
    let (status, _) = request(
        &app,
        "POST",
        "/api/pubsub/topics/t/messages",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_pubsub_delete_cascade() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"name": "tmp"});
    request(&app, "POST", "/api/pubsub/topics", Some(&token), Some(body)).await;

    let body = serde_json::json!({"body": "m1"});
    request(
        &app,
        "POST",
        "/api/pubsub/topics/tmp/messages",
        Some(&token),
        Some(body),
    )
    .await;

    let body = serde_json::json!({"body": "m2"});
    request(
        &app,
        "POST",
        "/api/pubsub/topics/tmp/messages",
        Some(&token),
        Some(body),
    )
    .await;

    request(&app, "DELETE", "/api/pubsub/topics/tmp", Some(&token), None).await;

    let (status, _) = request(
        &app,
        "GET",
        "/api/pubsub/topics/tmp/messages",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = test_app().await;
    let body = serde_json::json!({"email": "user@test.com", "password": "password123"});
    post_json(&app, "/api/auth/register", body).await;
    let body = serde_json::json!({"email": "user@test.com", "password": "wrongpassword"});
    let (status, _) = post_json(&app, "/api/auth/login", body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
