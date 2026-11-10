use super::*;

#[tokio::test]
async fn test_buckets_and_files() {
    let svr = Svr::start().await;
    let token = svr.register().await;

    // Create bucket
    let r = svr
        .post_auth("/api/buckets", &token, json!({"name": "photos"}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["name"], "photos");

    // List buckets
    let r = svr.get_auth("/api/buckets", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        r.json::<Value>().await.unwrap().as_array().unwrap().len(),
        1
    );

    // Get bucket
    let r = svr.get_auth("/api/buckets/photos", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(r.json::<Value>().await.unwrap()["name"] == "photos");

    // Upload file
    let content = b"hello world file content";
    let r = svr
        .post_multipart(
            "/api/buckets/photos/files",
            &token,
            "hello.txt",
            content,
            "text/plain",
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let file = r.json::<Value>().await.unwrap();
    let file_id = file["id"].as_str().unwrap().to_string();
    assert_eq!(file["filename"], "hello.txt");

    // List files
    let r = svr.get_auth("/api/buckets/photos/files", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    let files_resp = r.json::<Value>().await.unwrap();
    assert_eq!(files_resp["total"], 1);

    // Download file
    let r = svr
        .get_auth(&format!("/api/buckets/photos/files/{file_id}"), &token)
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let bytes = r.bytes().await.unwrap();
    assert_eq!(&bytes[..], content);

    // Upload to nonexistent bucket
    let r = svr
        .post_multipart(
            "/api/buckets/nonexistent/files",
            &token,
            "f.txt",
            b"x",
            "text/plain",
        )
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    // Duplicate bucket
    let r = svr
        .post_auth("/api/buckets", &token, json!({"name": "photos"}))
        .await;
    assert_eq!(r.status(), StatusCode::CONFLICT);

    // Delete file
    let r = svr
        .delete_auth(&format!("/api/buckets/photos/files/{file_id}"), &token)
        .await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    let r = svr
        .get_auth(&format!("/api/buckets/photos/files/{file_id}"), &token)
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    // Delete bucket
    let r = svr.delete_auth("/api/buckets/photos", &token).await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    let r = svr.get_auth("/api/buckets/photos", &token).await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
