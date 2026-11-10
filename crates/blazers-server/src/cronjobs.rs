use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use cron::Schedule;
use serde_json::Value;
use uuid::Uuid;

use crate::handlers::AppState;
use blazers_core::db;
use blazers_core::models::*;

pub const EVENT_CRONJOB_CREATE: &str = "cronjob.create";
pub const EVENT_CRONJOB_UPDATE: &str = "cronjob.update";
pub const EVENT_CRONJOB_DELETE: &str = "cronjob.delete";

pub fn webhook_cronjob_data(job: &CronJob) -> Value {
    serde_json::json!({"cronjob": job})
}

pub async fn start_cron_scheduler(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let jobs = {
                let conn = match state.db.get().await {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                db::list_cron_jobs(&conn).unwrap_or_default()
            };
            for job in jobs {
                if !job.is_active {
                    continue;
                }
                if job.schedule.is_empty() {
                    continue;
                }
                let schedule = match Schedule::from_str(&job.schedule) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let now = Utc::now();
                let last_run = if job.last_run_at.is_empty() {
                    now - chrono::Duration::hours(1)
                } else {
                    chrono::DateTime::parse_from_rfc3339(&job.last_run_at)
                        .map(|d| d.with_timezone(&chrono::Utc))
                        .unwrap_or(now - chrono::Duration::hours(1))
                };
                if let Some(next_run) = schedule.after(&last_run).next()
                    && next_run <= now
                {
                    execute_cron_job(Arc::clone(&state), job).await;
                }
            }
        }
    });
}

pub async fn execute_cron_job(state: Arc<AppState>, job: CronJob) {
    let started_at = Utc::now().to_rfc3339();
    let client = &state.http_client;

    let method = job.method.to_uppercase();
    let mut req_builder = match method.as_str() {
        "GET" => client.get(&job.command),
        "POST" => client.post(&job.command),
        "PUT" => client.put(&job.command),
        "PATCH" => client.patch(&job.command),
        "DELETE" => client.delete(&job.command),
        _ => client.get(&job.command),
    };

    if !job.headers.is_empty()
        && let Ok(headers) = serde_json::from_str::<Vec<Vec<String>>>(&job.headers)
    {
        for h in headers {
            if h.len() == 2 {
                req_builder = req_builder.header(&h[0], &h[1]);
            }
        }
    }

    if !job.body.is_empty() && (method == "POST" || method == "PUT" || method == "PATCH") {
        req_builder = req_builder.body(job.body.clone());
    }

    let (status, output, error) = match req_builder.send().await {
        Ok(resp) => {
            let sc = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            (sc as i64, body, String::new())
        }
        Err(e) => (0, String::new(), e.to_string()),
    };

    let finished_at = Utc::now().to_rfc3339();
    let started = chrono::DateTime::parse_from_rfc3339(&started_at)
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or(Utc::now());
    let finished = chrono::DateTime::parse_from_rfc3339(&finished_at)
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or(Utc::now());
    let duration_ms = (finished - started).num_milliseconds();

    let status_str = if (200..300).contains(&status) {
        "success"
    } else {
        "failure"
    };

    let log = CronJobLog {
        id: Uuid::new_v4().to_string().replace('-', ""),
        cronjob_id: job.id.clone(),
        started_at,
        finished_at,
        duration_ms,
        status: status_str.to_string(),
        output,
        error,
    };

    {
        let conn = match state.db.get().await {
            Ok(c) => c,
            Err(_) => return,
        };
        let _ = db::insert_cron_job_log(&conn, &log);
        let _ = db::update_cron_job_last_run(&conn, &job.id, &log.finished_at, status_str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::any;
    use deadpool::managed::Pool;
    use std::path::PathBuf;
    use tokio::net::TcpListener;

    fn sample_cronjob() -> CronJob {
        CronJob {
            id: "job-123".into(),
            name: "Test Job".into(),
            schedule: "*/5 * * * * *".into(),
            command: "https://example.com/hook".into(),
            method: "POST".into(),
            headers: r#"[["Authorization", "Bearer token"]]"#.into(),
            body: r#"{"key": "value"}"#.into(),
            is_active: true,
            last_run_at: "".into(),
            last_run_status: "".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn event_constants_are_correct() {
        assert_eq!(EVENT_CRONJOB_CREATE, "cronjob.create");
        assert_eq!(EVENT_CRONJOB_UPDATE, "cronjob.update");
        assert_eq!(EVENT_CRONJOB_DELETE, "cronjob.delete");
    }

    #[test]
    fn contains_cronjob_key() {
        let job = sample_cronjob();
        let data = webhook_cronjob_data(&job);
        assert!(data.get("cronjob").is_some());
    }

    #[test]
    fn preserves_name() {
        let job = sample_cronjob();
        let data = webhook_cronjob_data(&job);
        assert_eq!(data["cronjob"]["name"], "Test Job");
    }

    #[test]
    fn preserves_schedule() {
        let job = sample_cronjob();
        let data = webhook_cronjob_data(&job);
        assert_eq!(data["cronjob"]["schedule"], "*/5 * * * * *");
    }

    #[test]
    fn preserves_command() {
        let job = sample_cronjob();
        let data = webhook_cronjob_data(&job);
        assert_eq!(data["cronjob"]["command"], "https://example.com/hook");
    }

    #[test]
    fn preserves_is_active() {
        let job = sample_cronjob();
        let data = webhook_cronjob_data(&job);
        assert_eq!(data["cronjob"]["is_active"], true);
    }

    #[test]
    fn preserves_method() {
        let job = sample_cronjob();
        let data = webhook_cronjob_data(&job);
        assert_eq!(data["cronjob"]["method"], "POST");
    }

    #[test]
    fn webhook_cronjob_data_empty_values() {
        let job = CronJob {
            id: String::new(),
            name: String::new(),
            schedule: String::new(),
            command: String::new(),
            method: String::new(),
            headers: String::new(),
            body: String::new(),
            is_active: false,
            last_run_at: String::new(),
            last_run_status: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        let data = webhook_cronjob_data(&job);
        let cronjob = &data["cronjob"];
        assert_eq!(cronjob["is_active"], false);
        assert_eq!(cronjob["name"], "");
        assert_eq!(cronjob["id"], "");
        assert_eq!(cronjob["schedule"], "");
        assert_eq!(cronjob["command"], "");
        assert_eq!(cronjob["method"], "");
    }

    #[test]
    fn webhook_cronjob_data_inactive_job() {
        let job = CronJob {
            is_active: false,
            ..sample_cronjob()
        };
        let data = webhook_cronjob_data(&job);
        assert_eq!(data["cronjob"]["is_active"], false);
        assert_eq!(data["cronjob"]["name"], "Test Job");
    }

    #[test]
    fn webhook_cronjob_data_special_characters() {
        let job = CronJob {
            name: "Job with spéçïal & ch@rs!".into(),
            command: "https://example.com/hook?q=test&lang=en".into(),
            headers: r#"[["X-Custom", "val=1; type=test"]]"#.into(),
            body: r#"{"message": "hello \"world\""}"#.into(),
            ..sample_cronjob()
        };
        let data = webhook_cronjob_data(&job);
        assert_eq!(data["cronjob"]["name"], "Job with spéçïal & ch@rs!");
        assert_eq!(
            data["cronjob"]["command"],
            "https://example.com/hook?q=test&lang=en"
        );
        assert_eq!(
            data["cronjob"]["headers"],
            r#"[["X-Custom", "val=1; type=test"]]"#
        );
        assert_eq!(data["cronjob"]["body"], r#"{"message": "hello \"world\""}"#);
    }

    #[test]
    fn webhook_cronjob_data_all_fields_present() {
        let job = sample_cronjob();
        let data = webhook_cronjob_data(&job);
        let cronjob = &data["cronjob"];
        assert_eq!(cronjob["id"], "job-123");
        assert_eq!(cronjob["name"], "Test Job");
        assert_eq!(cronjob["schedule"], "*/5 * * * * *");
        assert_eq!(cronjob["command"], "https://example.com/hook");
        assert_eq!(cronjob["method"], "POST");
        assert_eq!(cronjob["is_active"], true);
        assert_eq!(cronjob["last_run_at"], "");
        assert_eq!(cronjob["last_run_status"], "");
        assert_eq!(cronjob["created_at"], "2025-01-01T00:00:00Z");
        assert_eq!(cronjob["updated_at"], "2025-01-01T00:00:00Z");
    }

    #[test]
    fn webhook_cronjob_data_headers_and_body() {
        let job = CronJob {
            headers: r#"[["X-API-Key", "secret123"]]"#.into(),
            body: r#"{"data": "test"}"#.into(),
            ..sample_cronjob()
        };
        let data = webhook_cronjob_data(&job);
        assert_eq!(
            data["cronjob"]["headers"],
            r#"[["X-API-Key", "secret123"]]"#
        );
        assert_eq!(data["cronjob"]["body"], r#"{"data": "test"}"#);
    }

    // --- Helpers ---

    fn create_test_env() -> (Arc<AppState>, PathBuf) {
        let tmp_dir = std::env::temp_dir().join(format!("blazers-test-cron-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let db_path = tmp_dir.join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        db::migrate_db(&conn).unwrap();
        drop(conn);
        let pool: Pool<db::ConnectionManager> =
            deadpool::managed::Pool::builder(db::ConnectionManager { path: db_path })
                .build()
                .unwrap();
        let state = Arc::new(AppState {
            db: pool,
            storage_dir: tmp_dir.clone(),
            http_client: reqwest::Client::new(),
            secrets_key: vec![1, 2, 3, 4],
            ws_hub: crate::websocket::new_ws_hub(),
            cache: Arc::new(crate::cache::CacheStore::new()),
            sse_hub: crate::notification::new_sse_hub(),
            log_hub: crate::log::new_log_hub(),
            pubsub_hub: crate::pubsub::new_pubsub_hub(),
            rate_limiter: Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
        });
        (state, tmp_dir)
    }

    async fn test_server(status: u16, body: &'static str) -> std::net::SocketAddr {
        let app = axum::Router::new().route(
            "/hook",
            any(move || async move { (axum::http::StatusCode::from_u16(status).unwrap(), body) }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        addr
    }

    fn make_job(id: &str, method: &str, command: &str, headers: &str, body: &str) -> CronJob {
        CronJob {
            id: id.into(),
            name: format!("{method}-test"),
            schedule: "* * * * * *".into(),
            command: command.into(),
            method: method.into(),
            headers: headers.into(),
            body: body.into(),
            is_active: true,
            last_run_at: String::new(),
            last_run_status: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    // --- execute_cron_job tests ---

    #[tokio::test]
    async fn execute_cron_job_get_success() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "ok").await;
        let job = make_job("cg-get", "GET", &format!("http://{addr}/hook"), "", "");
        {
            let conn = state.db.get().await.unwrap();
            db::insert_cron_job(
                &conn,
                &job.id,
                &job.name,
                &job.schedule,
                &job.command,
                &job.method,
                &job.headers,
                &job.body,
            )
            .unwrap();
        }
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-get").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        assert_eq!(logs[0].output, "ok");
        let updated = db::get_cron_job(&conn, "cg-get").unwrap().unwrap();
        assert_eq!(updated.last_run_status, "success");
        assert!(!updated.last_run_at.is_empty());
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_post_with_body() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "posted").await;
        let job = make_job(
            "cg-post",
            "POST",
            &format!("http://{addr}/hook"),
            r#"[["Content-Type", "application/json"]]"#,
            r#"{"key": "value"}"#,
        );
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-post").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        assert_eq!(logs[0].output, "posted");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_put_with_body() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "put-ok").await;
        let job = make_job(
            "cg-put",
            "PUT",
            &format!("http://{addr}/hook"),
            "",
            r#"{"updated": true}"#,
        );
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-put").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_patch_with_body() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "patched").await;
        let job = make_job(
            "cg-patch",
            "PATCH",
            &format!("http://{addr}/hook"),
            "",
            r#"{"patched": true}"#,
        );
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-patch").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_delete() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "deleted").await;
        let job = make_job("cg-del", "DELETE", &format!("http://{addr}/hook"), "", "");
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-del").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_unknown_method_defaults_to_get() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "default-get").await;
        let job = make_job(
            "cg-unknown",
            "OPTIONS",
            &format!("http://{addr}/hook"),
            "",
            "",
        );
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-unknown").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        assert_eq!(logs[0].output, "default-get");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_with_custom_headers() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "headers-ok").await;
        let job = make_job(
            "cg-headers",
            "GET",
            &format!("http://{addr}/hook"),
            r#"[["X-Custom", "val123"], ["Authorization", "Bearer tok"]]"#,
            "",
        );
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-headers").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_error_status() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(500, "server-error").await;
        let job = make_job("cg-err", "GET", &format!("http://{addr}/hook"), "", "");
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-err").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "failure");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn execute_cron_job_network_error() {
        let (state, tmp_dir) = create_test_env();
        let job = make_job("cg-net", "GET", "http://127.0.0.1:1/hook", "", "");
        {
            let conn = state.db.get().await.unwrap();
            db::insert_cron_job(
                &conn,
                &job.id,
                &job.name,
                &job.schedule,
                &job.command,
                &job.method,
                &job.headers,
                &job.body,
            )
            .unwrap();
        }
        execute_cron_job(Arc::clone(&state), job).await;
        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "cg-net").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "failure");
        assert!(!logs[0].error.is_empty());
        // On network error, status is 0
        let updated = db::get_cron_job(&conn, "cg-net").unwrap().unwrap();
        assert_eq!(updated.last_run_status, "failure");
        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    // --- start_cron_scheduler tests ---

    #[tokio::test]
    async fn start_cron_scheduler_executes_due_job() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "scheduler-ok").await;

        {
            let conn = state.db.get().await.unwrap();
            db::insert_cron_job(
                &conn,
                "sched-test",
                "scheduler test",
                "* * * * * *",
                &format!("http://{addr}/hook"),
                "GET",
                "",
                "",
            )
            .unwrap();
        }

        start_cron_scheduler(Arc::clone(&state)).await;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "sched-test").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");

        let updated = db::get_cron_job(&conn, "sched-test").unwrap().unwrap();
        assert_eq!(updated.last_run_status, "success");
        assert!(!updated.last_run_at.is_empty());

        std::fs::remove_dir_all(&tmp_dir).ok();
    }

    #[tokio::test]
    async fn start_cron_scheduler_skips_inactive_jobs() {
        let (state, tmp_dir) = create_test_env();
        let addr = test_server(200, "skipped").await;

        {
            let conn = state.db.get().await.unwrap();
            db::insert_cron_job(
                &conn,
                "sched-inactive",
                "inactive test",
                "* * * * * *",
                &format!("http://{addr}/hook"),
                "GET",
                "",
                "",
            )
            .unwrap();
            // Mark job as inactive
            db::update_cron_job(
                &conn,
                "sched-inactive",
                "inactive test",
                "* * * * * *",
                &format!("http://{addr}/hook"),
                "GET",
                "",
                "",
                false,
            )
            .unwrap();
        }

        start_cron_scheduler(Arc::clone(&state)).await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let conn = state.db.get().await.unwrap();
        let logs = db::list_cron_job_logs(&conn, "sched-inactive").unwrap();
        assert_eq!(logs.len(), 0);

        std::fs::remove_dir_all(&tmp_dir).ok();
    }
}
