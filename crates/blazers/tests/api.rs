#![allow(clippy::zombie_processes)]
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};

fn find_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct Svr {
    child: Option<Child>,
    _tmp: PathBuf,
    base: String,
    cli: Client,
}

impl Svr {
    async fn start() -> Self {
        let port = find_port();
        let base = format!("http://127.0.0.1:{port}");
        let tmp = std::env::temp_dir().join(format!("blazers-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).ok();

        let child = Command::new(env!("CARGO_BIN_EXE_blazers"))
            .env("PORT", port.to_string())
            .env("BLAZERS_DATA", &tmp)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to start blazers server");

        let cli = Client::new();
        let health = format!("{base}/api/health");
        for _ in 0..30 {
            if let Ok(r) = cli.get(&health).send().await
                && r.status().is_success()
            {
                return Self {
                    child: Some(child),
                    _tmp: tmp,
                    base,
                    cli,
                };
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        panic!("blazers server did not start within timeout");
    }

    fn url(&self, p: &str) -> String {
        format!("{}{}", self.base, p)
    }

    async fn get(&self, p: &str) -> reqwest::Response {
        self.cli.get(self.url(p)).send().await.unwrap()
    }

    async fn get_auth(&self, p: &str, token: &str) -> reqwest::Response {
        self.cli
            .get(self.url(p))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap()
    }

    async fn post(&self, p: &str, body: Value) -> reqwest::Response {
        self.cli.post(self.url(p)).json(&body).send().await.unwrap()
    }

    async fn post_auth(&self, p: &str, token: &str, body: Value) -> reqwest::Response {
        self.cli
            .post(self.url(p))
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    async fn patch_auth(&self, p: &str, token: &str, body: Value) -> reqwest::Response {
        self.cli
            .patch(self.url(p))
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    async fn delete_auth(&self, p: &str, token: &str) -> reqwest::Response {
        self.cli
            .delete(self.url(p))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap()
    }

    async fn post_multipart(
        &self,
        p: &str,
        token: &str,
        filename: &str,
        content: &[u8],
        mime: &str,
    ) -> reqwest::Response {
        let part = reqwest::multipart::Part::bytes(content.to_vec())
            .file_name(filename.to_string())
            .mime_str(mime)
            .unwrap();
        let form = reqwest::multipart::Form::new().part("file", part);
        self.cli
            .post(self.url(p))
            .header("Authorization", format!("Bearer {token}"))
            .multipart(form)
            .send()
            .await
            .unwrap()
    }

    async fn register(&self) -> String {
        let r = self
            .post(
                "/api/auth/register",
                json!({"email": "admin@test.com", "password": "admin123"}),
            )
            .await;
        assert_eq!(
            r.status(),
            StatusCode::OK,
            "register failed: {:?}",
            r.text().await
        );
        let r = self
            .post(
                "/api/auth/login",
                json!({"email": "admin@test.com", "password": "admin123"}),
            )
            .await;
        assert_eq!(r.status(), StatusCode::OK);
        let login: Value = r.json().await.unwrap();
        let token = login["token"].as_str().unwrap().to_string();
        let user_id = login["user"]["id"].as_str().unwrap().to_string();

        // Grant the user global admin so they can create collections etc.
        let r = self
            .post_auth(
                "/api/permissions",
                &token,
                json!({"user_id": user_id, "collection": "*", "role": "admin"}),
            )
            .await;
        assert_eq!(
            r.status(),
            StatusCode::CREATED,
            "grant permission failed: {:?}",
            r.text().await
        );

        token
    }
}

impl Drop for Svr {
    fn drop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        let _ = std::fs::remove_dir_all(&self._tmp);
    }
}

#[tokio::test]
async fn test_health_and_auth() {
    let svr = Svr::start().await;

    // Health
    let r = svr.get("/api/health").await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["status"], "ok");

    // Register
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": "user@test.com", "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    assert_eq!(body["email"], "user@test.com");
    assert!(!body["id"].as_str().unwrap().is_empty());

    // Duplicate register
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": "user@test.com", "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::CONFLICT);

    // Login
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "user@test.com", "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    assert!(!body["token"].as_str().unwrap().is_empty());
    assert_eq!(body["user"]["email"], "user@test.com");

    // Bad password
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "user@test.com", "password": "wrong"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    // Missing fields
    let r = svr.post("/api/auth/register", json!({})).await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Auth guard — /api/collections without token
    let r = svr.get("/api/collections").await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    // Invalid token
    let r = svr.get_auth("/api/collections", "invalid-token").await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
}

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
