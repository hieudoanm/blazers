use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn list_cron_jobs(conn: &Connection) -> Result<Vec<CronJob>> {
    let mut stmt = conn
        .prepare("SELECT id, name, schedule, command, method, headers, body, is_active, last_run_at, last_run_status, created_at, updated_at FROM _cronjobs ORDER BY name")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CronJob {
                id: row.get(0)?,
                name: row.get(1)?,
                schedule: row.get(2)?,
                command: row.get(3)?,
                method: row.get(4)?,
                headers: row.get(5)?,
                body: row.get(6)?,
                is_active: row.get::<_, i64>(7)? != 0,
                last_run_at: row.get(8)?,
                last_run_status: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut jobs = Vec::new();
    for row in rows {
        jobs.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(jobs)
}

pub fn get_cron_job(conn: &Connection, id: &str) -> Result<Option<CronJob>> {
    let mut stmt = conn
        .prepare("SELECT id, name, schedule, command, method, headers, body, is_active, last_run_at, last_run_status, created_at, updated_at FROM _cronjobs WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(CronJob {
                id: row.get(0)?,
                name: row.get(1)?,
                schedule: row.get(2)?,
                command: row.get(3)?,
                method: row.get(4)?,
                headers: row.get(5)?,
                body: row.get(6)?,
                is_active: row.get::<_, i64>(7)? != 0,
                last_run_at: row.get(8)?,
                last_run_status: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(j)) => Ok(Some(j)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn insert_cron_job(
    conn: &Connection,
    id: &str,
    name: &str,
    schedule: &str,
    command: &str,
    method: &str,
    headers: &str,
    body: &str,
) -> Result<CronJob> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _cronjobs (id, name, schedule, command, method, headers, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, name, schedule, command, method, headers, body, now, now],
    )
    .map_err(|e| AppError::Internal(format!("insert cron job: {e}")))?;
    Ok(CronJob {
        id: id.to_string(),
        name: name.to_string(),
        schedule: schedule.to_string(),
        command: command.to_string(),
        method: method.to_string(),
        headers: headers.to_string(),
        body: body.to_string(),
        is_active: true,
        last_run_at: String::new(),
        last_run_status: String::new(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn update_cron_job(
    conn: &Connection,
    id: &str,
    name: &str,
    schedule: &str,
    command: &str,
    method: &str,
    headers: &str,
    body: &str,
    is_active: bool,
) -> Result<CronJob> {
    let now = Utc::now().to_rfc3339();
    let active = if is_active { 1 } else { 0 };
    conn.execute(
        "UPDATE _cronjobs SET name = ?1, schedule = ?2, command = ?3, method = ?4, headers = ?5, body = ?6, is_active = ?7, updated_at = ?8 WHERE id = ?9",
        params![name, schedule, command, method, headers, body, active, now, id],
    )
    .map_err(|e| AppError::Internal(format!("update cron job: {e}")))?;
    get_cron_job(conn, id)?
        .ok_or_else(|| AppError::Internal("cron job not found after update".into()))
}

pub fn delete_cron_job(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM _cronjob_logs WHERE cronjob_id = ?1",
        params![id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM _cronjobs WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn update_cron_job_last_run(
    conn: &Connection,
    id: &str,
    last_run_at: &str,
    status: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE _cronjobs SET last_run_at = ?1, last_run_status = ?2, updated_at = ?3 WHERE id = ?4",
        params![last_run_at, status, now, id],
    )
    .map_err(|e| AppError::Internal(format!("update cron job last run: {e}")))?;
    Ok(())
}

pub fn list_cron_job_logs(conn: &Connection, cronjob_id: &str) -> Result<Vec<CronJobLog>> {
    let mut stmt = conn
        .prepare("SELECT id, cronjob_id, started_at, finished_at, duration_ms, status, output, error FROM _cronjob_logs WHERE cronjob_id = ?1 ORDER BY started_at DESC LIMIT 50")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![cronjob_id], |row| {
            Ok(CronJobLog {
                id: row.get(0)?,
                cronjob_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                duration_ms: row.get(4)?,
                status: row.get(5)?,
                output: row.get(6)?,
                error: row.get(7)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(logs)
}

pub fn insert_cron_job_log(conn: &Connection, log: &CronJobLog) -> Result<()> {
    conn.execute(
        "INSERT INTO _cronjob_logs (id, cronjob_id, started_at, finished_at, duration_ms, status, output, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![log.id, log.cronjob_id, log.started_at, log.finished_at, log.duration_ms, log.status, log.output, log.error],
    )
    .map_err(|e| AppError::Internal(format!("insert cron job log: {e}")))?;
    Ok(())
}
