use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    /// Prints only the message, so a handler can surface the cause to a caller
    /// without also leaking the HTTP status it maps to.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            AppError::BadRequest(m)
            | AppError::NotFound(m)
            | AppError::Conflict(m)
            | AppError::Unauthorized(m)
            | AppError::Forbidden(m)
            | AppError::Internal(m) => m,
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, m),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        (code, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub schema: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub data: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordsPage {
    pub records: Vec<Record>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    #[serde(default = "default_schema")]
    pub schema: String,
}

fn default_schema() -> String {
    "{}".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CreateRecordRequest {
    pub id: Option<String>,
    #[serde(default = "default_data")]
    pub data: Value,
}

fn default_data() -> Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecordRequest {
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RecordListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollectionRequest {
    pub schema: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
    #[serde(default)]
    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub bucket: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct FilesPage {
    pub files: Vec<FileRecord>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    #[serde(default)]
    pub secret: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookLog {
    pub id: String,
    pub webhook_id: String,
    pub event: String,
    pub url: String,
    pub request_body: String,
    pub response_status: i64,
    pub response_body: String,
    #[serde(default)]
    pub error: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub scope: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub scope: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSecretRequest {
    pub name: Option<String>,
    pub value: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub secret: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WSConnection {
    pub id: String,
    pub remote_addr: String,
    pub path: String,
    pub user_agent: String,
    pub connected_at: String,
    pub disconnected_at: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WSMessage {
    pub id: String,
    pub connection_id: String,
    pub direction: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WSSendRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub ttl: i64,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SetCacheRequest {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
    pub ttl: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(rename = "type")]
    pub ntype: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(rename = "type")]
    pub ntype: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubTopic {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubMessage {
    pub id: String,
    pub topic_id: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePubSubTopicRequest {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePubSubMessageRequest {
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub level: String,
    pub message: String,
    #[serde(default)]
    pub meta: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub meta: String,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::json;

    #[test]
    fn default_schema_value() {
        assert_eq!(default_schema(), "{}");
    }

    #[test]
    fn default_data_value() {
        assert_eq!(default_data(), json!({}));
    }

    #[test]
    fn app_error_bad_request() {
        let resp = AppError::BadRequest("bad".into()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn app_error_not_found() {
        let resp = AppError::NotFound("nf".into()).into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn app_error_conflict() {
        let resp = AppError::Conflict("conflict".into()).into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn app_error_unauthorized() {
        let resp = AppError::Unauthorized("unauth".into()).into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn app_error_forbidden() {
        let resp = AppError::Forbidden("forbid".into()).into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn app_error_internal() {
        let resp = AppError::Internal("err".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn record_round_trip() {
        let record = Record {
            id: "rec_1".into(),
            data: json!({"name": "test"}),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&record).unwrap();
        let d: Record = serde_json::from_value(v).unwrap();
        assert_eq!(record.id, d.id);
        assert_eq!(record.data, d.data);
    }

    #[test]
    fn records_page_round_trip() {
        let record = Record {
            id: "rec_1".into(),
            data: json!({"x": 1}),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let page = RecordsPage {
            records: vec![record],
            total: 1,
            page: 1,
            per_page: 20,
            total_pages: 1,
        };
        let v = serde_json::to_value(&page).unwrap();
        let d: RecordsPage = serde_json::from_value(v).unwrap();
        assert_eq!(page.total, d.total);
        assert_eq!(page.records.len(), d.records.len());
    }

    #[test]
    fn login_response_serialization() {
        let user = UserResponse {
            id: "u1".into(),
            email: "a@b.com".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let resp = LoginResponse {
            token: "tok_1".into(),
            user,
        };
        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v["token"], "tok_1");
        assert_eq!(v["user"]["email"], "a@b.com");
    }

    #[test]
    fn bucket_round_trip() {
        let bucket = Bucket {
            name: "b".into(),
            is_public: true,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&bucket).unwrap();
        let d: Bucket = serde_json::from_value(v).unwrap();
        assert_eq!(bucket.name, d.name);
        assert_eq!(bucket.is_public, d.is_public);
    }

    #[test]
    fn file_record_round_trip() {
        let file = FileRecord {
            id: "f1".into(),
            bucket: "b".into(),
            filename: "t.txt".into(),
            mime_type: "text/plain".into(),
            size: 100,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&file).unwrap();
        let d: FileRecord = serde_json::from_value(v).unwrap();
        assert_eq!(file.id, d.id);
        assert_eq!(file.size, d.size);
    }

    #[test]
    fn cache_entry_round_trip() {
        let entry = CacheEntry {
            key: "k".into(),
            value: "v".into(),
            ttl: 3600,
            expires_at: "2024-01-02T00:00:00Z".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&entry).unwrap();
        let d: CacheEntry = serde_json::from_value(v).unwrap();
        assert_eq!(entry.key, d.key);
        assert_eq!(entry.ttl, d.ttl);
    }

    #[test]
    fn notification_round_trip() {
        let n = Notification {
            id: "n1".into(),
            title: "Hi".into(),
            body: "Body".into(),
            ntype: "info".into(),
            is_read: false,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&n).unwrap();
        let d: Notification = serde_json::from_value(v).unwrap();
        assert_eq!(n.id, d.id);
        assert_eq!(n.ntype, d.ntype);
    }

    #[test]
    fn notification_serializes_ntype_as_type() {
        let n = Notification {
            id: "n2".into(),
            title: "T".into(),
            body: "B".into(),
            ntype: "alert".into(),
            is_read: true,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&n).unwrap();
        assert_eq!(v["type"], "alert");
        assert!(v.get("ntype").is_none());
    }

    #[test]
    fn create_notification_request_deserializes_type() {
        let req: CreateNotificationRequest =
            serde_json::from_value(json!({"title": "Test", "type": "info"})).unwrap();
        assert_eq!(req.title, "Test");
        assert_eq!(req.ntype, "info");
    }

    #[test]
    fn create_collection_request_defaults_schema() {
        let req: CreateCollectionRequest = serde_json::from_value(json!({"name": "c"})).unwrap();
        assert_eq!(req.name, "c");
        assert_eq!(req.schema, "{}");
    }

    #[test]
    fn create_record_request_defaults_data() {
        let req: CreateRecordRequest = serde_json::from_value(json!({"id": "r1"})).unwrap();
        assert_eq!(req.id.unwrap(), "r1");
        assert_eq!(req.data, json!({}));
    }

    #[test]
    fn register_request_defaults_empty_strings() {
        let req: RegisterRequest = serde_json::from_value(json!({})).unwrap();
        assert_eq!(req.email, "");
        assert_eq!(req.password, "");
    }

    #[test]
    fn empty_records_page() {
        let page = RecordsPage {
            records: vec![],
            total: 0,
            page: 1,
            per_page: 20,
            total_pages: 0,
        };
        let v = serde_json::to_value(&page).unwrap();
        let d: RecordsPage = serde_json::from_value(v).unwrap();
        assert!(d.records.is_empty());
        assert_eq!(d.total, 0);
        assert_eq!(d.total_pages, 0);
    }

    #[test]
    fn cache_entry_zero_ttl() {
        let entry = CacheEntry {
            key: "k".into(),
            value: "v".into(),
            ttl: 0,
            expires_at: "2024-01-01T00:00:00Z".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&entry).unwrap();
        let d: CacheEntry = serde_json::from_value(v).unwrap();
        assert_eq!(d.ttl, 0);
    }

    #[test]
    fn notification_empty_body() {
        let n = Notification {
            id: "n3".into(),
            title: "Empty".into(),
            body: "".into(),
            ntype: "warn".into(),
            is_read: false,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&n).unwrap();
        let d: Notification = serde_json::from_value(v).unwrap();
        assert_eq!(d.body, "");
    }
}
