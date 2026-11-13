use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Minimal HTTP stub: `/ok` returns a record patch, anything else 500s.
async fn stub_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let (status, body) = if req.starts_with("POST /ok") {
                    (
                        "200 OK",
                        r#"{"record":{"injected":"yes","status":"hooked"}}"#,
                    )
                } else {
                    ("500 Internal Server Error", r#"{"error":"nope"}"#)
                };
                let resp = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            });
        }
    });
    format!("http://127.0.0.1:{port}")
}

#[tokio::test]
async fn test_record_before_create_hook() {
    let svr = Svr::start().await;
    let admin = svr.register().await;
    let base = stub_server().await;

    svr.post_auth("/api/collections", &admin, json!({"name": "hooked"}))
        .await;
    // patch hook: injects fields into the record data
    let r = svr
        .post_auth(
            "/api/webhooks",
            &admin,
            json!({
                "name": "patch", "url": format!("{base}/ok"),
                "events": ["record.before_create"]
            }),
        )
        .await;
    assert!(r.status().is_success());
    let wh_id = r.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let r = svr
        .post_auth(
            "/api/collections/hooked/records",
            &admin,
            json!({"data": {"a": 1}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let rec = r.json::<Value>().await.unwrap();
    assert_eq!(rec["data"]["injected"], "yes");
    assert_eq!(rec["data"]["status"], "hooked");
    assert_eq!(rec["data"]["a"], 1);

    // failing hook cancels the write with 400
    let r = svr
        .patch_auth(
            &format!("/api/webhooks/{wh_id}"),
            &admin,
            json!({"url": format!("{base}/fail")}),
        )
        .await;
    assert!(r.status().is_success());

    let r = svr
        .post_auth(
            "/api/collections/hooked/records",
            &admin,
            json!({"data": {"a": 2}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // and nothing was written
    let r = svr
        .get_auth("/api/collections/hooked/records", &admin)
        .await;
    assert_eq!(r.json::<Value>().await.unwrap()["total"], 1);
}
