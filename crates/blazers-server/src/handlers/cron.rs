use super::*;

pub async fn list_cron_jobs(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<CronJob>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let jobs = db::list_cron_jobs(&conn)?;
    Ok(Json(jobs))
}

pub async fn create_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<CreateCronJobRequest>,
) -> std::result::Result<(StatusCode, Json<CronJob>), AppError> {
    extract_claims(&headers)?;
    if req.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if req.schedule.is_empty() {
        return Err(AppError::BadRequest("schedule is required".into()));
    }
    if req.command.is_empty() {
        return Err(AppError::BadRequest("command is required".into()));
    }
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let method = if req.method.is_empty() {
        "GET".to_string()
    } else {
        req.method
    };
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let job = db::insert_cron_job(
        &conn,
        &id,
        &req.name,
        &req.schedule,
        &req.command,
        &method,
        &req.headers,
        &req.body,
    )?;
    webhook::dispatch_event(
        &state,
        crate::cronjobs::EVENT_CRONJOB_CREATE,
        crate::cronjobs::webhook_cronjob_data(&job),
    );
    Ok((StatusCode::CREATED, Json(job)))
}

pub async fn get_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<CronJob>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let job =
        db::get_cron_job(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;
    Ok(Json(job))
}

pub async fn update_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCronJobRequest>,
) -> std::result::Result<Json<CronJob>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing =
        db::get_cron_job(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?;

    let name = req.name.unwrap_or(existing.name);
    let schedule = req.schedule.unwrap_or(existing.schedule);
    let command = req.command.unwrap_or(existing.command);
    let method = req.method.unwrap_or(existing.method);
    let headers = req.headers.unwrap_or(existing.headers);
    let body = req.body.unwrap_or(existing.body);
    let is_active = req.is_active.unwrap_or(existing.is_active);

    let job = db::update_cron_job(
        &conn, &id, &name, &schedule, &command, &method, &headers, &body, is_active,
    )?;
    webhook::dispatch_event(
        &state,
        crate::cronjobs::EVENT_CRONJOB_UPDATE,
        crate::cronjobs::webhook_cronjob_data(&job),
    );
    Ok(Json(job))
}

pub async fn delete_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_cron_job(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let job = existing.unwrap();
    webhook::dispatch_event(
        &state,
        crate::cronjobs::EVENT_CRONJOB_DELETE,
        crate::cronjobs::webhook_cronjob_data(&job),
    );
    db::delete_cron_job(&conn, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn run_cron_job(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Value>, AppError> {
    extract_claims(&headers)?;
    let job = {
        let conn = state
            .db
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::get_cron_job(&conn, &id)?.ok_or_else(|| AppError::NotFound("not found".into()))?
    };
    crate::cronjobs::execute_cron_job(state, job).await;
    Ok(Json(serde_json::json!({"status": "triggered"})))
}

pub async fn list_cron_job_logs(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<CronJobLog>>, AppError> {
    extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::get_cron_job(&conn, &id)?;
    if existing.is_none() {
        return Err(AppError::NotFound("not found".into()));
    }
    let logs = db::list_cron_job_logs(&conn, &id)?;
    Ok(Json(logs))
}
