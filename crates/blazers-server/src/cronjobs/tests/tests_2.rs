use super::*;

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
