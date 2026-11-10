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
mod tests;
