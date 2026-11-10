use super::*;

#[tokio::test]
async fn test_webhook_events_dispatch_on_record_create() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    // Use a test HTTP server to verify webhook delivery
    use std::sync::Arc as StdArc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let received = StdArc::new(AtomicUsize::new(0));
    let received_clone = StdArc::clone(&received);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let recv = StdArc::clone(&received_clone);
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
                let (reader, mut writer) = tokio::io::split(stream);
                let mut buf_reader = BufReader::new(reader);
                let mut line = String::new();
                loop {
                    line.clear();
                    match buf_reader.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(_) => {
                            if line.trim().is_empty() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
                let _ = writer.write_all(resp.as_bytes()).await;
                recv.fetch_add(1, Ordering::SeqCst);
            });
        }
    });

    // Create a collection first
    let body = serde_json::json!({"name": "items"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    // Create webhook pointing at test server
    let wh_body = serde_json::json!({
        "name": "test",
        "url": format!("http://{addr}/hook"),
        "events": ["record.create"]
    });
    let (status, _) = request(&app, "POST", "/api/webhooks", Some(&token), Some(wh_body)).await;
    assert_eq!(status, StatusCode::OK);

    // Create a record (triggers webhook)
    let rec_body = serde_json::json!({"data": {"hello": "world"}});
    let (status, _) = request(
        &app,
        "POST",
        "/api/collections/items/records",
        Some(&token),
        Some(rec_body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Wait for webhook delivery
    for _ in 0..10 {
        if received.load(Ordering::SeqCst) > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        received.load(Ordering::SeqCst) > 0,
        "webhook was not delivered"
    );
}

// --- Secret Tests ---

#[tokio::test]
async fn test_secrets_crud() {
    let app = test_app().await;
    let token = register_token(&app).await;

    // Create
    let body = serde_json::json!({"name": "db_pass", "value": "s3cret!", "scope": "db"});
    let (status, value) = request(&app, "POST", "/api/secrets", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "create secret");
    let id = value["id"].as_str().unwrap_or("").to_string();
    assert!(!id.is_empty());
    assert_eq!(value["name"], "db_pass");
    assert_eq!(value["scope"], "db");

    // List (value should not be present)
    let (status, value) = get_json(&app, "/api/secrets", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let list = value.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert!(
        list[0].get("value").is_none(),
        "value should not be in list"
    );

    // Get (decrypted value)
    let (status, value) = request(
        &app,
        "GET",
        &format!("/api/secrets/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["value"], "s3cret!");

    // Update
    let body = serde_json::json!({"name": "db_pass_v2", "value": "newpass", "scope": "prod"});
    let (status, _) = request(
        &app,
        "PATCH",
        &format!("/api/secrets/{id}"),
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, value) = request(
        &app,
        "GET",
        &format!("/api/secrets/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["value"], "newpass");

    // Delete
    let (status, _) = request(
        &app,
        "DELETE",
        &format!("/api/secrets/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(
        &app,
        "GET",
        &format!("/api/secrets/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_secrets_validation() {
    let app = test_app().await;
    let token = register_token(&app).await;

    // Missing name
    let body = serde_json::json!({"value": "x"});
    let (status, _) = request(&app, "POST", "/api/secrets", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Not found
    let (status, _) = request(&app, "GET", "/api/secrets/nonexistent", Some(&token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Update not found
    let body = serde_json::json!({"name": "x", "value": "y"});
    let (status, _) = request(
        &app,
        "PATCH",
        "/api/secrets/nonexistent",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
