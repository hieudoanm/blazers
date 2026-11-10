use super::*;

#[tokio::test]
async fn test_notifications_and_logs() {
    let svr = Svr::start().await;
    let token = svr.register().await;

    // Create notification
    let r = svr
        .post_auth(
            "/api/notifications",
            &token,
            json!({"title": "Test", "body": "Body", "type": "info"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let n = r.json::<Value>().await.unwrap();
    let nid = n["id"].as_str().unwrap().to_string();
    assert_eq!(n["title"], "Test");
    assert!(!n["is_read"].as_bool().unwrap());

    // List
    let r = svr.get_auth("/api/notifications", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        r.json::<Value>().await.unwrap().as_array().unwrap().len(),
        1
    );

    // Mark read
    let r = svr
        .patch_auth(&format!("/api/notifications/{nid}"), &token, json!({}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(
        r.json::<Value>().await.unwrap()["is_read"]
            .as_bool()
            .unwrap()
    );

    // Delete
    let r = svr
        .delete_auth(&format!("/api/notifications/{nid}"), &token)
        .await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Clear
    let r = svr
        .post_auth(
            "/api/notifications",
            &token,
            json!({"title": "X", "type": "warning"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let r = svr.delete_auth("/api/notifications", &token).await;
    assert_eq!(r.status(), StatusCode::OK);

    let r = svr.get_auth("/api/notifications", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        r.json::<Value>().await.unwrap().as_array().unwrap().len(),
        0
    );

    // Create log
    let r = svr
        .post_auth(
            "/api/logs",
            &token,
            json!({"level": "info", "message": "test log entry"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        r.json::<Value>().await.unwrap()["message"],
        "test log entry"
    );

    // List logs
    let r = svr.get_auth("/api/logs", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        r.json::<Value>().await.unwrap().as_array().unwrap().len(),
        1
    );

    // Clear logs
    let r = svr.delete_auth("/api/logs", &token).await;
    assert_eq!(r.status(), StatusCode::OK);

    let r = svr.get_auth("/api/logs", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        r.json::<Value>().await.unwrap().as_array().unwrap().len(),
        0
    );
}

#[tokio::test]
async fn test_openapi_spec() {
    let svr = Svr::start().await;

    let r = svr.get("/api/openapi.json").await;
    assert_eq!(r.status(), StatusCode::OK);
    let spec = r.json::<Value>().await.unwrap();
    assert_eq!(spec["openapi"], "3.0.0");
    assert!(spec.get("paths").is_some());
    assert!(spec["paths"].get("/api/health").is_some());
}
