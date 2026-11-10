use super::*;

#[tokio::test]
async fn test_invalid_token() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/collections", Some("invalid-token")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_bucket_crud() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"name": "avatars"});
    let (status, value) = request(&app, "POST", "/api/buckets", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["name"], "avatars");

    let (status, value) = get_json(&app, "/api/buckets", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let buckets = value.as_array().unwrap();
    assert_eq!(buckets.len(), 1);

    let (status, value) = request(&app, "GET", "/api/buckets/avatars", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["name"], "avatars");

    let (status, _) = request(&app, "DELETE", "/api/buckets/avatars", Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(&app, "GET", "/api/buckets/avatars", Some(&token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_file_upload_download_delete() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"name": "photos"});
    let (status, _) = request(&app, "POST", "/api/buckets", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let content = b"Hello, World!";
    let (status, value) = upload_file(&app, "photos", "hello.txt", content, &token).await;
    assert_eq!(status, StatusCode::OK);
    let file_id = value["id"].as_str().unwrap_or("").to_string();
    assert!(!file_id.is_empty());
    assert_eq!(value["filename"], "hello.txt");
    assert_eq!(value["bucket"], "photos");

    let (status, value) = get_json(&app, "/api/buckets/photos/files", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let files = value["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/buckets/photos/files/{file_id}"))
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(&body_bytes[..], content);
    assert!(
        ct.starts_with("text/plain") || ct == "application/octet-stream",
        "expected text/plain or application/octet-stream, got {ct:?}"
    );

    let (status, _) = request(
        &app,
        "DELETE",
        &format!("/api/buckets/photos/files/{file_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(
        &app,
        "GET",
        &format!("/api/buckets/photos/files/{file_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_file_no_bucket() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let (status, _) = upload_file(&app, "nonexistent", "f.txt", b"data", &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = get_json(&app, "/api/buckets", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_bucket_duplicate() {
    let app = test_app().await;
    let token = register_token(&app).await;

    let body = serde_json::json!({"name": "dup"});
    let (status, _) = request(
        &app,
        "POST",
        "/api/buckets",
        Some(&token),
        Some(body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = request(&app, "POST", "/api/buckets", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}
