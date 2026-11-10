use super::*;

#[tokio::test]
async fn test_websocket_crud() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let (status, value) = get_json(&app, "/api/websockets", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(value.as_array().unwrap().is_empty());
    let (status, _) = get_json(&app, "/api/websockets/messages", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_websocket_broadcast() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"content": "hello all"});
    let (status, value) = request(
        &app,
        "POST",
        "/api/websockets/broadcast",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(value.get("sent").is_some());
    let (status, _value) = get_json(&app, "/api/websockets/messages", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_websocket_not_found() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let (status, _) = get_json(&app, "/api/websockets/nonexistent", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = request(
        &app,
        "DELETE",
        "/api/websockets/nonexistent",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_websocket_send_not_found() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"content": "hello"});
    let (status, _) = request(
        &app,
        "POST",
        "/api/websockets/nonexistent/send",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_websocket_messages_list() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let (status, _value) =
        get_json(&app, "/api/websockets/nonexistent/messages", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cache_ttl() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let body = serde_json::json!({"key": "temp", "value": "data", "ttl": 1});
    let (status, _) = request(&app, "POST", "/api/cache", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = get_json(&app, "/api/cache/temp", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let (status, _) = get_json(&app, "/api/cache/temp", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cache_stats() {
    let app = test_app().await;
    let token = register_token(&app).await;
    let (status, value) = get_json(&app, "/api/cache/stats", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(value.get("total_entries").is_some());
    assert!(value.get("expired_entries").is_some());
    let body = serde_json::json!({"key": "a", "value": "1"});
    let (status, _) = request(&app, "POST", "/api/cache", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, value) = get_json(&app, "/api/cache/stats", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["total_entries"].as_u64(), Some(1));
}

#[tokio::test]
async fn test_cache_stats_auth_required() {
    let app = test_app().await;
    let (status, _) = get_json(&app, "/api/cache/stats", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_signature() {
    let (app, state) = test_app_with_state().await;
    let token = register_admin(&app, &state).await;

    use std::sync::Arc as StdArc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let got_sig = StdArc::new(AtomicBool::new(false));
    let got_sig_clone = StdArc::clone(&got_sig);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let got = StdArc::clone(&got_sig_clone);
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
                        if line.to_lowercase().starts_with("x-webhook-signature-256") {
                            got.store(true, Ordering::SeqCst);
                        }
                    }
                    Err(_) => break,
                }
            }
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
            let _ = writer.write_all(resp.as_bytes()).await;
        });
    });

    let wh_body = serde_json::json!({
        "name": "sig test",
        "url": format!("http://{addr}/hook"),
        "events": ["record.create"],
        "secret": "mysecret"
    });
    let (status, _) = request(&app, "POST", "/api/webhooks", Some(&token), Some(wh_body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"name": "sig_test"});
    let (status, _) = request(&app, "POST", "/api/collections", Some(&token), Some(body)).await;
    assert_eq!(status, StatusCode::OK);

    let body = serde_json::json!({"data": {"x": 1}});
    let (status, _) = request(
        &app,
        "POST",
        "/api/collections/sig_test/records",
        Some(&token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert!(
        got_sig.load(Ordering::SeqCst),
        "should have received signature header"
    );
}
