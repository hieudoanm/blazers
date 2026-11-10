use super::*;

#[tokio::test]
async fn test_webhook_crud() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({
        "name": "my hook",
        "url": "https://example.com/hook",
        "events": ["record.create", "record.delete"]
    });
    let (status, value) = request(&app, "POST", "/api/webhooks", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let hook_id = value["id"].as_str().unwrap_or("").to_string();
    assert!(!hook_id.is_empty(), "expected webhook id");

    let (status, value) = get_json(&app, "/api/webhooks", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let hooks = value.as_array().unwrap();
    assert_eq!(hooks.len(), 1);

    let (status, value) = request(
        &app,
        "GET",
        &format!("/api/webhooks/{hook_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["name"], "my hook");

    let body = serde_json::json!({"name": "updated hook", "is_active": false});
    let (status, value) = request(
        &app,
        "PATCH",
        &format!("/api/webhooks/{hook_id}"),
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["name"], "updated hook");
    assert!(!value["is_active"].as_bool().unwrap_or(true));

    let (status, _) = request(
        &app,
        "DELETE",
        &format!("/api/webhooks/{hook_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(
        &app,
        "GET",
        &format!("/api/webhooks/{hook_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_webhook_validation() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let cases = [
        (
            serde_json::json!({"url": "https://example.com/h", "events": ["record.create"]}),
            "name is required",
        ),
        (
            serde_json::json!({"name": "test", "events": ["record.create"]}),
            "url is required",
        ),
        (
            serde_json::json!({"name": "test", "url": "https://example.com/h"}),
            "at least one event is required",
        ),
        (
            serde_json::json!({"name": "test", "url": "https://example.com/h", "events": ["invalid.event"]}),
            "unknown event",
        ),
    ];
    for (i, (body, expected)) in cases.iter().enumerate() {
        let (status, value) = request(
            &app,
            "POST",
            "/api/webhooks",
            Some(&token),
            Some(body.clone()),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "case {i}: expected BAD_REQUEST for '{expected}'"
        );
        let err = value["error"].as_str().unwrap_or("");
        assert!(
            err.contains(expected),
            "case {i}: expected '{expected}' in '{err}'"
        );
    }
}

#[tokio::test]
async fn test_webhook_logs() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({
        "name": "log test",
        "url": "https://example.com/hook",
        "events": ["record.create"]
    });
    let (status, value) = request(&app, "POST", "/api/webhooks", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let hook_id = value["id"].as_str().unwrap_or("").to_string();

    let (status, value) = request(
        &app,
        "GET",
        &format!("/api/webhooks/{hook_id}/logs"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(value.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_webhook_auth_required() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/webhooks", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_event_validation() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({
        "name": "bad events",
        "url": "https://example.com/h",
        "events": ["record.create", "bogus"]
    });
    let (status, value) = request(&app, "POST", "/api/webhooks", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        value["error"]
            .as_str()
            .unwrap_or("")
            .contains("unknown event")
    );
}
