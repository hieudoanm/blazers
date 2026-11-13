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

    async fn put_auth(&self, p: &str, token: &str, body: Value) -> reqwest::Response {
        self.cli
            .put(self.url(p))
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

mod auth;
mod auth_flows;
mod auth_mfa;
mod batch;
mod cache;
mod collections;
mod features;
mod hooks;
mod misc;
mod storage;
mod users;
