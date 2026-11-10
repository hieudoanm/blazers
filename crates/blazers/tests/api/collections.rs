use super::*;

#[tokio::test]
async fn test_crud_collections_and_records() {
    let svr = Svr::start().await;
    let token = svr.register().await;

    // Create collection
    let r = svr
        .post_auth("/api/collections", &token, json!({"name": "articles"}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["name"], "articles");

    // List collections
    let r = svr.get_auth("/api/collections", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    let cols = r.json::<Value>().await.unwrap();
    assert!(
        cols.as_array()
            .unwrap()
            .iter()
            .any(|c| c["name"] == "articles")
    );

    // Get collection
    let r = svr.get_auth("/api/collections/articles", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["name"], "articles");

    // Duplicate collection
    let r = svr
        .post_auth("/api/collections", &token, json!({"name": "articles"}))
        .await;
    assert_eq!(r.status(), StatusCode::CONFLICT);

    // Create record
    let r = svr
        .post_auth(
            "/api/collections/articles/records",
            &token,
            json!({"data": {"title": "Hello", "views": 42}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let rec = r.json::<Value>().await.unwrap();
    let rec_id = rec["id"].as_str().unwrap().to_string();
    assert_eq!(rec["data"]["title"], "Hello");

    // List records
    let r = svr
        .get_auth(
            "/api/collections/articles/records?page=1&per_page=20",
            &token,
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let page = r.json::<Value>().await.unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["records"].as_array().unwrap().len(), 1);

    // Get record
    let r = svr
        .get_auth(
            &format!("/api/collections/articles/records/{rec_id}"),
            &token,
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["id"], rec_id);

    // Update record
    let r = svr
        .patch_auth(
            &format!("/api/collections/articles/records/{rec_id}"),
            &token,
            json!({"data": {"title": "Updated", "views": 99}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["data"]["title"], "Updated");

    // Record not found
    let r = svr
        .get_auth("/api/collections/articles/records/nonexistent", &token)
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    // Delete record
    let r = svr
        .delete_auth(
            &format!("/api/collections/articles/records/{rec_id}"),
            &token,
        )
        .await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    let r = svr
        .get_auth(
            &format!("/api/collections/articles/records/{rec_id}"),
            &token,
        )
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    // Delete collection
    let r = svr.delete_auth("/api/collections/articles", &token).await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    let r = svr.get_auth("/api/collections/articles", &token).await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
