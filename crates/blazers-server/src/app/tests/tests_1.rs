use super::*;

#[tokio::test]
async fn test_health() {
    let app = test_app().await;
    let (status, value) = get_json(&app, "/api/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["status"], "ok");
}

#[tokio::test]
async fn test_auth_register_and_login() {
    let app = test_app().await;
    let body = serde_json::json!({"email": "test@example.com", "password": "password123"});
    let (status, value) = post_json(&app, "/api/auth/register", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["email"], "test@example.com");

    let body = serde_json::json!({"email": "test@example.com", "password": "password123"});
    let (status, value) = post_json(&app, "/api/auth/login", body).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!value["token"].as_str().unwrap_or("").is_empty());
    assert_eq!(value["user"]["email"], "test@example.com");
}

#[tokio::test]
async fn test_auth_requires_token() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/collections", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_collection_crud() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    let body = serde_json::json!({"name": "posts", "schema": "{\"title\":\"string\"}"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, value) = get_json(&app, "/api/collections", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let cols = value.as_array().unwrap();
    // one data collection + the synthetic `users` auth collection
    assert_eq!(cols.len(), 2);
    assert!(
        cols.iter()
            .any(|c| c["name"] == "users" && c["type"] == "auth")
    );

    let (status, value) = request(&app, "GET", "/api/collections/posts", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["name"], "posts");

    let (status, _) = request(&app, "DELETE", "/api/collections/posts", Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(&app, "GET", "/api/collections/posts", Some(&token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_record_crud() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    let body = serde_json::json!({"name": "items"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"data": {"name": "item1", "value": 42}});
    let (status, value) = request(
        &app,
        "POST",
        "/api/collections/items/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let rec_id = value["id"].as_str().unwrap_or("").to_string();
    assert!(!rec_id.is_empty());

    let (status, value) = request(
        &app,
        "GET",
        &format!("/api/collections/items/records/{rec_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["id"], rec_id);

    let (status, value) = get_json(
        &app,
        "/api/collections/items/records?page=1&per_page=20",
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["total"], 1);

    let body = serde_json::json!({"data": {"name": "item1-updated", "value": 99}});
    let (status, _) = request(
        &app,
        "PATCH",
        &format!("/api/collections/items/records/{rec_id}"),
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = request(
        &app,
        "DELETE",
        &format!("/api/collections/items/records/{rec_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(
        &app,
        "GET",
        &format!("/api/collections/items/records/{rec_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_duplicate_email() {
    let app = test_app().await;
    let body = serde_json::json!({"email": "dup@test.com", "password": "password123"});
    let (status, _) = post_json(&app, "/api/auth/register", body.clone()).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = post_json(&app, "/api/auth/register", body).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_missing_auth_fields() {
    let app = test_app().await;
    let cases = vec![
        (serde_json::json!({}), StatusCode::BAD_REQUEST),
        (
            serde_json::json!({"email": "test@test.com"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            serde_json::json!({"password": "test"}),
            StatusCode::BAD_REQUEST,
        ),
        (
            serde_json::json!({"email": "test@test.com", "password": "123"}),
            StatusCode::BAD_REQUEST,
        ),
    ];
    for (body, expected) in cases {
        let (status, _) = post_json(&app, "/api/auth/register", body).await;
        assert_eq!(status, expected);
    }
}

#[tokio::test]
async fn test_collection_not_found() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    let (status, _) = request(
        &app,
        "GET",
        "/api/collections/nonexistent",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let body = serde_json::json!({"data": {}});
    let (status, _) = request(
        &app,
        "POST",
        "/api/collections/nonexistent/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
