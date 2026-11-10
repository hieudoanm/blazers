use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub command: String,
    pub method: String,
    pub headers: String,
    pub body: String,
    pub is_active: bool,
    pub last_run_at: String,
    pub last_run_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CronJobLog {
    pub id: String,
    pub cronjob_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: i64,
    pub status: String,
    pub output: String,
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCronJobRequest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub schedule: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub headers: String,
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCronJobRequest {
    pub name: Option<String>,
    pub schedule: Option<String>,
    pub command: Option<String>,
    pub method: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub is_active: Option<bool>,
}
