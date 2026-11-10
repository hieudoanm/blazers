use super::*;

#[tokio::test]
async fn test_webhook_no_delivery_on_inactive() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

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

    let wh_body = serde_json::json!({
        "name": "inactive test",
        "url": format!("http://{addr}/hook"),
        "events": ["record.create"]
    });
    let (status, value) = request(&app, "POST", "/api/webhooks", Some(&token), Some(wh_body)).await;
    assert_eq!(status, StatusCode::OK);
    let hook_id = value["id"].as_str().unwrap_or("").to_string();

    let body = serde_json::json!({"is_active": false});
    let (status, _) = request(
        &app,
        "PATCH",
        &format!("/api/webhooks/{hook_id}"),
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"name": "inact_test"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"data": {"x": 1}});
    let (status, _) = request(
        &app,
        "POST",
        "/api/collections/inact_test/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert_eq!(
        received.load(Ordering::SeqCst),
        0,
        "inactive webhook should not deliver"
    );
}

#[tokio::test]
async fn test_webhook_update_not_found() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"name": "nope"});
    let (status, _) = request(
        &app,
        "PATCH",
        "/api/webhooks/nonexistent",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_webhook_delete_not_found() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let (status, _) = request(
        &app,
        "DELETE",
        "/api/webhooks/nonexistent",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cron_jobs_run_and_logs() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let (reader, mut writer) = tokio::io::split(stream);
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            if buf_reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                break;
            }
            if line.trim().is_empty() {
                break;
            }
        }
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\npong";
        let _ = writer.write_all(resp.as_bytes()).await;
    });

    let body = serde_json::json!({
        "name": "test ping",
        "schedule": "*/5 * * * *",
        "command": format!("http://{addr}/ping"),
        "method": "GET"
    });
    let (status, value) = request(&app, "POST", "/api/cronjobs", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = value["id"].as_str().unwrap_or("").to_string();

    let (status, _) = request(
        &app,
        "POST",
        &format!("/api/cronjobs/{id}/run"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let (status, value) = get_json(&app, &format!("/api/cronjobs/{id}/logs"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let logs = value.as_array().unwrap();
    assert!(!logs.is_empty());
    assert_eq!(logs[0]["status"], "success");
}

#[tokio::test]
async fn test_notification_get() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"title": "Get Me", "body": "Find me", "type": "info"});
    let (status, value) =
        request(&app, "POST", "/api/notifications", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let id = value["id"].as_str().unwrap_or("").to_string();

    let (status, value) = get_json(&app, &format!("/api/notifications/{id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["title"], "Get Me");
    assert_eq!(value["body"], "Find me");
}

#[tokio::test]
async fn test_notification_get_not_found() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let (status, _) = get_json(&app, "/api/notifications/nonexistent", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
