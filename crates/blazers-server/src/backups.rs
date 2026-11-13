//! Scheduled SQLite backups. Configured via `_settings`:
//!   `backup_schedule` — cron expression (e.g. `0 3 * * *`); unset/empty disables.
//!   `backup_dir`      — output directory (default `$BLAZERS_DATA/backups`).
//!   `backup_keep`     — max snapshots to retain (default 7, 0 = keep all).
//!
//! The scheduler polls settings every 30s, so changes take effect without a
//! restart. Backups use `VACUUM INTO` like the on-demand `/api/backup`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use blazers_core::db;

use crate::handlers::AppState;

async fn setting(state: &Arc<AppState>, key: &str) -> Option<String> {
    let conn = state.db.get().await.ok()?;
    db::get_setting(&conn, key).ok()?
}

/// Runs one backup now: VACUUM INTO a timestamped file, then prune.
/// Returns the written path.
pub async fn run_backup(state: &Arc<AppState>) -> Result<PathBuf, String> {
    let dir = match setting(state, "backup_dir").await {
        Some(d) if !d.is_empty() => PathBuf::from(d),
        _ => state.storage_dir.join("backups"),
    };
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("create backup dir: {e}"))?;

    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let path = dir.join(format!("blazers-backup-{stamp}.db"));

    {
        let conn = state.db.get().await.map_err(|e| e.to_string())?;
        conn.execute(
            "VACUUM INTO ?1",
            rusqlite::params![path.to_str().unwrap_or("")],
        )
        .map_err(|e| format!("backup: {e}"))?;
    }
    prune(&dir, keep_count(state).await).await;
    Ok(path)
}

async fn keep_count(state: &Arc<AppState>) -> usize {
    setting(state, "backup_keep")
        .await
        .and_then(|v| v.parse().ok())
        .unwrap_or(7)
}

/// Deletes oldest `blazers-backup-*.db` files beyond `keep` (0 = keep all).
async fn prune(dir: &std::path::Path, keep: usize) {
    if keep == 0 {
        return;
    }
    let mut entries: Vec<PathBuf> = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            let p = entry.path();
            if p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("blazers-backup-") && n.ends_with(".db"))
            {
                entries.push(p);
            }
        }
    }
    entries.sort();
    let excess = entries.len().saturating_sub(keep);
    for path in entries.into_iter().take(excess) {
        let _ = tokio::fs::remove_file(path).await;
    }
}

/// POST /api/backup/run — run a backup immediately (admin only).
pub async fn handle_run_backup(
    headers: axum::http::HeaderMap,
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
) -> std::result::Result<axum::Json<serde_json::Value>, blazers_core::models::AppError> {
    crate::rbac::require_role_pub(&state, &headers, "*", "admin").await?;
    let path = run_backup(&state)
        .await
        .map_err(blazers_core::models::AppError::Internal)?;
    Ok(axum::Json(serde_json::json!({
        "path": path.display().to_string(),
    })))
}

/// Long-running scheduler; spawn once at server start.
pub async fn scheduler(state: Arc<AppState>) {
    let mut next_fire: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_schedule = String::new();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let schedule = setting(&state, "backup_schedule")
            .await
            .unwrap_or_default()
            .trim()
            .to_string();
        if schedule != last_schedule {
            last_schedule = schedule.clone();
            next_fire = None;
        }
        if schedule.is_empty() {
            continue;
        }
        let Ok(parsed) = schedule.parse::<cron::Schedule>() else {
            eprintln!("[backups] invalid backup_schedule cron expression: {schedule}");
            continue;
        };
        if next_fire.is_none() {
            next_fire = parsed.upcoming(chrono::Utc).next();
        }
        let Some(fire) = next_fire else { continue };
        if chrono::Utc::now() >= fire {
            match run_backup(&state).await {
                Ok(path) => eprintln!("[backups] wrote {}", path.display()),
                Err(e) => eprintln!("[backups] failed: {e}"),
            }
            next_fire = parsed.upcoming(chrono::Utc).next();
        }
    }
}
