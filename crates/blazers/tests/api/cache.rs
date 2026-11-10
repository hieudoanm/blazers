use super::*;

#[tokio::test]
async fn test_cache_operations() {
    let svr = Svr::start().await;
    let token = svr.register().await;

    // Set cache
    let r = svr
        .post_auth(
            "/api/cache",
            &token,
            json!({"key": "greeting", "value": "hello world"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["value"], "hello world");

    // Overwrite
    let r = svr
        .post_auth(
            "/api/cache",
            &token,
            json!({"key": "greeting", "value": "updated"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // Get
    let r = svr.get_auth("/api/cache/greeting", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["value"], "updated");

    // List
    let r = svr.get_auth("/api/cache", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(
        !r.json::<Value>()
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );

    // Stats
    let r = svr.get_auth("/api/cache/stats", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    let stats = r.json::<Value>().await.unwrap();
    assert!(stats.get("total_entries").is_some());

    // Not found
    let r = svr.get_auth("/api/cache/nonexistent", &token).await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    // Missing value
    let r = svr
        .post_auth("/api/cache", &token, json!({"key": "k"}))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Delete
    let r = svr.delete_auth("/api/cache/greeting", &token).await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Flush
    let r = svr.delete_auth("/api/cache", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
}
