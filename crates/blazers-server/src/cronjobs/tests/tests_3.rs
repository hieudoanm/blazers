use super::*;

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
