use crate::db::*;
use crate::models::*;
use rusqlite::Connection;

#[test]
fn test_cron_job_crud() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let job = insert_cron_job(
        &conn,
        "cj1",
        "daily_task",
        "0 0 * * *",
        "echo hi",
        "GET",
        "",
        "",
    )
    .unwrap();
    assert_eq!(job.name, "daily_task");
    assert!(job.is_active);

    let jobs = list_cron_jobs(&conn).unwrap();
    assert_eq!(jobs.len(), 1);

    let fetched = get_cron_job(&conn, "cj1").unwrap().unwrap();
    assert_eq!(fetched.schedule, "0 0 * * *");

    let updated = update_cron_job(
        &conn,
        "cj1",
        "nightly",
        "0 0 * * 0",
        "echo bye",
        "POST",
        "{}",
        "{}",
        false,
    )
    .unwrap();
    assert!(!updated.is_active);
    assert_eq!(updated.name, "nightly");

    delete_cron_job(&conn, "cj1").unwrap();
    assert!(get_cron_job(&conn, "cj1").unwrap().is_none());
}

#[test]
fn test_cron_job_last_run() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    insert_cron_job(&conn, "cj2", "test_job", "* * * * *", "cmd", "GET", "", "").unwrap();
    update_cron_job_last_run(&conn, "cj2", "2024-01-01T00:00:00Z", "success").unwrap();

    let job = get_cron_job(&conn, "cj2").unwrap().unwrap();
    assert_eq!(job.last_run_at, "2024-01-01T00:00:00Z");
    assert_eq!(job.last_run_status, "success");
}

#[test]
fn test_cron_job_log() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();

    let log = CronJobLog {
        id: "cjl1".into(),
        cronjob_id: "cj1".into(),
        started_at: "2024-01-01T00:00:00Z".into(),
        finished_at: "2024-01-01T01:00:00Z".into(),
        duration_ms: 3600000,
        status: "success".into(),
        output: "done".into(),
        error: String::new(),
    };
    insert_cron_job_log(&conn, &log).unwrap();

    let logs = list_cron_job_logs(&conn, "cj1").unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].status, "success");
}

// --- Secret CRUD ---

#[test]
fn test_update_cron_job_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    let err = update_cron_job(
        &conn,
        "nojob",
        "n",
        "* * * * *",
        "cmd",
        "GET",
        "",
        "",
        false,
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Internal(_)));
}

#[test]
fn test_get_cron_job_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(get_cron_job(&conn, "nonexistent").unwrap().is_none());
}

#[test]
fn test_list_cron_job_logs_empty() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    assert!(list_cron_job_logs(&conn, "nonexistent").unwrap().is_empty());
}
