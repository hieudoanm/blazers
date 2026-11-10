use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use rusqlite::{Connection, params};

use crate::handlers::AppState;
use blazers_core::models::*;

pub fn require_role(
    conn: &Connection,
    user_id: &str,
    collection: &str,
    required: &str,
) -> std::result::Result<(), AppError> {
    let mut stmt = conn
        .prepare("SELECT role FROM _permissions WHERE user_id = ?1 AND (collection = ?2 OR collection = '*')")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let roles: Vec<String> = stmt
        .query_map(params![user_id, collection], |row| row.get(0))
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    for role in &roles {
        if role == "admin" || role == required {
            return Ok(());
        }
    }
    if collection != "*" {
        let mut stmt = conn
            .prepare("SELECT role FROM _permissions WHERE user_id = ?1 AND collection = '*'")
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let global_roles: Vec<String> = stmt
            .query_map(params![user_id], |row| row.get(0))
            .map_err(|e| AppError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        for role in &global_roles {
            if role == "admin" || role == required {
                return Ok(());
            }
        }
    }
    Err(AppError::Forbidden("insufficient permissions".into()))
}

pub async fn handle_list_permissions(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<Vec<Permission>>, AppError> {
    crate::handlers::extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut stmt = conn
        .prepare("SELECT id, user_id, collection, role, created_at, updated_at FROM _permissions ORDER BY collection")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Permission {
                id: row.get(0)?,
                user_id: row.get(1)?,
                collection: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut perms = Vec::new();
    for row in rows {
        perms.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(Json(perms))
}

pub async fn handle_create_permission(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<PermissionRequest>,
) -> std::result::Result<(StatusCode, Json<Permission>), AppError> {
    crate::handlers::extract_claims(&headers)?;
    if req.user_id.is_empty() || req.collection.is_empty() || req.role.is_empty() {
        return Err(AppError::BadRequest(
            "user_id, collection, and role are required".into(),
        ));
    }
    if !matches!(req.role.as_str(), "admin" | "editor" | "viewer") {
        return Err(AppError::BadRequest(
            "role must be admin, editor, or viewer".into(),
        ));
    }
    let id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute(
        "INSERT INTO _permissions (id, user_id, collection, role) VALUES (?1, ?2, ?3, ?4)",
        params![id, req.user_id, req.collection, req.role],
    )
    .map_err(|e| AppError::Internal(format!("insert permission: {e}")))?;
    let perm = find_permission_by_id(&conn, &id)?
        .ok_or_else(|| AppError::Internal("permission not found after create".into()))?;
    Ok((StatusCode::CREATED, Json(perm)))
}

pub async fn handle_delete_permission(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
    crate::handlers::extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let n = conn
        .execute("DELETE FROM _permissions WHERE id = ?1", params![id])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound("permission not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn find_permission_by_id(
    conn: &Connection,
    id: &str,
) -> std::result::Result<Option<Permission>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, user_id, collection, role, created_at, updated_at FROM _permissions WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(Permission {
                id: row.get(0)?,
                user_id: row.get(1)?,
                collection: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(p)) => Ok(Some(p)),
        _ => Ok(None),
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub user_id: String,
    pub collection: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PermissionRequest {
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub collection: String,
    #[serde(default)]
    pub role: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_round_trip() {
        let p = Permission {
            id: "p1".into(),
            user_id: "u1".into(),
            collection: "c1".into(),
            role: "admin".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&p).unwrap();
        let d: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, d.id);
        assert_eq!(p.user_id, d.user_id);
        assert_eq!(p.collection, d.collection);
        assert_eq!(p.role, d.role);
    }

    #[test]
    fn permission_request_defaults_empty() {
        let req: PermissionRequest = serde_json::from_str("{}").unwrap();
        assert_eq!(req.user_id, "");
        assert_eq!(req.collection, "");
        assert_eq!(req.role, "");
    }

    fn create_perms_table(conn: &rusqlite::Connection) {
        conn.execute_batch(
            "CREATE TABLE _permissions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                collection TEXT NOT NULL,
                role TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT '',
                UNIQUE(user_id, collection)
            );",
        )
        .unwrap();
    }

    #[test]
    fn test_require_role_admin_global() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', '*', 'admin')",
            [],
        )
        .unwrap();
        assert!(require_role(&conn, "u1", "any_collection", "viewer").is_ok());
    }

    #[test]
    fn test_require_role_exact_match() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'docs', 'editor')",
            [],
        )
        .unwrap();
        assert!(require_role(&conn, "u1", "docs", "editor").is_ok());
    }

    #[test]
    fn test_require_role_admin_exact() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'docs', 'admin')",
            [],
        )
        .unwrap();
        assert!(require_role(&conn, "u1", "docs", "viewer").is_ok());
    }

    #[test]
    fn test_require_role_insufficient() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'other', 'viewer')",
            [],
        )
        .unwrap();
        let result = require_role(&conn, "u1", "test", "viewer");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }

    #[test]
    fn test_require_role_no_permissions() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        let result = require_role(&conn, "u1", "test", "viewer");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }

    #[test]
    fn test_require_role_global_viewer() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', '*', 'viewer')",
            [],
        )
        .unwrap();
        assert!(require_role(&conn, "u1", "some_collection", "viewer").is_ok());
    }

    #[test]
    fn test_require_role_different_user() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', '*', 'admin')",
            [],
        )
        .unwrap();
        let result = require_role(&conn, "u2", "test", "viewer");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }

    #[test]
    fn test_find_permission_by_id_found() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'c1', 'editor')",
            [],
        )
        .unwrap();
        let found = find_permission_by_id(&conn, "p1").unwrap().unwrap();
        assert_eq!(found.user_id, "u1");
        assert_eq!(found.role, "editor");
        assert_eq!(found.collection, "c1");
    }

    #[test]
    fn test_find_permission_by_id_not_found() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        create_perms_table(&conn);
        assert!(
            find_permission_by_id(&conn, "nonexistent")
                .unwrap()
                .is_none()
        );
    }

    // --- handler tests ---

    fn test_token() -> String {
        use chrono::Utc;
        use jsonwebtoken::{EncodingKey, Header, encode};
        let claims = crate::auth::Claims {
            user_id: "test-user-id".into(),
            email: "test@example.com".into(),
            exp: Utc::now().timestamp() as usize + 3600,
            iat: Utc::now().timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev-secret-change-in-production".into())
                    .as_bytes(),
            ),
        )
        .unwrap()
    }

    async fn test_rbac_app() -> (axum::Router, std::sync::Arc<crate::handlers::AppState>) {
        use axum::routing::{delete, get};
        use blazers_core::db::ConnectionManager;
        use deadpool::managed::Pool;

        let tmp_dir = std::env::temp_dir().join(format!("blazers-rbac-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir).ok();
        let db_path = tmp_dir.join("test.db");
        let pool: Pool<ConnectionManager> = Pool::builder(ConnectionManager { path: db_path })
            .build()
            .unwrap();
        let conn = pool.get().await.unwrap();
        blazers_core::db::migrate_db(&conn).expect("migrate");
        drop(conn);
        let state = std::sync::Arc::new(crate::handlers::AppState {
            db: pool,
            storage_dir: tmp_dir,
            http_client: reqwest::Client::new(),
            secrets_key: vec![1, 2, 3, 4],
            ws_hub: crate::websocket::new_ws_hub(),
            cache: std::sync::Arc::new(crate::cache::CacheStore::new()),
            sse_hub: crate::notification::new_sse_hub(),
            log_hub: crate::log::new_log_hub(),
            pubsub_hub: crate::pubsub::new_pubsub_hub(),
            rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
        });
        let app = axum::Router::new()
            .route(
                "/api/permissions",
                get(handle_list_permissions).post(handle_create_permission),
            )
            .route("/api/permissions/{id}", delete(handle_delete_permission))
            .with_state(state.clone());
        (app, state)
    }

    async fn request_with_auth(
        app: &axum::Router,
        method: axum::http::Method,
        path: &str,
        token: &str,
        body: Option<serde_json::Value>,
    ) -> (axum::http::StatusCode, serde_json::Value) {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let req_builder = Request::builder()
            .method(method)
            .uri(path)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json");
        let b = body
            .map(|v| Body::from(serde_json::to_string(&v).unwrap()))
            .unwrap_or_else(Body::empty);
        let req = req_builder.body(b).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
        (status, value)
    }

    #[tokio::test]
    async fn test_handle_list_permissions_empty() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let (status, value) = request_with_auth(
            &app,
            axum::http::Method::GET,
            "/api/permissions",
            &token,
            None,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(value.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_handle_create_permission() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
        let (status, value) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert!(!value["id"].as_str().unwrap_or("").is_empty());
        assert_eq!(value["user_id"], "user1");
        assert_eq!(value["collection"], "docs");
        assert_eq!(value["role"], "viewer");
    }

    #[tokio::test]
    async fn test_handle_list_permissions_after_create() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
        let (status, _) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::CREATED);
        let (status, value) = request_with_auth(
            &app,
            axum::http::Method::GET,
            "/api/permissions",
            &token,
            None,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
        let perms = value.as_array().unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0]["user_id"], "user1");
    }

    #[tokio::test]
    async fn test_handle_delete_permission() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
        let (status, value) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::CREATED);
        let perm_id = value["id"].as_str().unwrap().to_string();
        let (status, _) = request_with_auth(
            &app,
            axum::http::Method::DELETE,
            &format!("/api/permissions/{perm_id}"),
            &token,
            None,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_handle_delete_permission_not_found() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let (status, value) = request_with_auth(
            &app,
            axum::http::Method::DELETE,
            "/api/permissions/nonexistent-id",
            &token,
            None,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
        assert_eq!(value["error"], "permission not found");
    }

    #[tokio::test]
    async fn test_handle_create_permission_missing_fields() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let body = serde_json::json!({"user_id": "", "collection": "docs", "role": "viewer"});
        let (status, value) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert!(value["error"].as_str().unwrap_or("").contains("required"));
    }

    #[tokio::test]
    async fn test_handle_create_permission_invalid_role() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let body =
            serde_json::json!({"user_id": "user1", "collection": "docs", "role": "superadmin"});
        let (status, value) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert!(
            value["error"]
                .as_str()
                .unwrap_or("")
                .contains("role must be")
        );
    }

    #[tokio::test]
    async fn test_handle_permissions_requires_auth() {
        let (app, _state) = test_rbac_app().await;
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let req = Request::builder()
            .uri("/api/permissions")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

        let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/permissions")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

        let req = Request::builder()
            .method("DELETE")
            .uri("/api/permissions/some-id")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_create_permission_duplicate() {
        let (app, _state) = test_rbac_app().await;
        let token = test_token();
        let body = serde_json::json!({"user_id": "user1", "collection": "docs", "role": "viewer"});
        let (status, _) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body.clone()),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::CREATED);
        let (status, _) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_handle_permissions_db_error_returns_internal_error() {
        use blazers_core::db::ConnectionManager;
        use deadpool::Runtime::Tokio1;
        use deadpool::managed::Pool;

        let pool: Pool<ConnectionManager> = Pool::builder(ConnectionManager {
            path: std::path::PathBuf::from(":memory:"),
        })
        .max_size(1)
        .runtime(Tokio1)
        .create_timeout(Some(std::time::Duration::from_millis(1)))
        .wait_timeout(Some(std::time::Duration::from_millis(1)))
        .build()
        .unwrap();
        let _conn = pool.get().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let state = std::sync::Arc::new(crate::handlers::AppState {
            db: pool,
            storage_dir: std::env::temp_dir(),
            http_client: reqwest::Client::new(),
            secrets_key: vec![1, 2, 3, 4],
            ws_hub: crate::websocket::new_ws_hub(),
            cache: std::sync::Arc::new(crate::cache::CacheStore::new()),
            sse_hub: crate::notification::new_sse_hub(),
            log_hub: crate::log::new_log_hub(),
            pubsub_hub: crate::pubsub::new_pubsub_hub(),
            rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
        });
        let app = axum::Router::new()
            .route(
                "/api/permissions",
                axum::routing::get(handle_list_permissions),
            )
            .with_state(state.clone());
        let token = test_token();
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;
        let req = Request::builder()
            .uri("/api/permissions")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    async fn exhausted_pool_app() -> (axum::Router, std::sync::Arc<crate::handlers::AppState>) {
        use blazers_core::db::ConnectionManager;
        use deadpool::Runtime::Tokio1;
        use deadpool::managed::Pool;

        let pool: Pool<ConnectionManager> = Pool::builder(ConnectionManager {
            path: std::path::PathBuf::from(":memory:"),
        })
        .max_size(1)
        .runtime(Tokio1)
        .create_timeout(Some(std::time::Duration::from_millis(1)))
        .wait_timeout(Some(std::time::Duration::from_millis(1)))
        .build()
        .unwrap();
        let _conn = pool.get().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let state = std::sync::Arc::new(crate::handlers::AppState {
            db: pool,
            storage_dir: std::env::temp_dir(),
            http_client: reqwest::Client::new(),
            secrets_key: vec![1, 2, 3, 4],
            ws_hub: crate::websocket::new_ws_hub(),
            cache: std::sync::Arc::new(crate::cache::CacheStore::new()),
            sse_hub: crate::notification::new_sse_hub(),
            log_hub: crate::log::new_log_hub(),
            pubsub_hub: crate::pubsub::new_pubsub_hub(),
            rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(200.0, 100.0)),
        });
        let app = axum::Router::new()
            .route(
                "/api/permissions",
                axum::routing::get(handle_list_permissions).post(handle_create_permission),
            )
            .route(
                "/api/permissions/{id}",
                axum::routing::delete(handle_delete_permission),
            )
            .with_state(state.clone());
        (app, state)
    }

    #[tokio::test]
    async fn test_handle_create_permission_db_error() {
        let (app, _state) = exhausted_pool_app().await;
        let token = test_token();
        let body = serde_json::json!({"user_id": "u1", "collection": "c1", "role": "viewer"});
        let (status, _) = request_with_auth(
            &app,
            axum::http::Method::POST,
            "/api/permissions",
            &token,
            Some(body),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_handle_delete_permission_db_error() {
        let (app, _state) = exhausted_pool_app().await;
        let token = test_token();
        let (status, _) = request_with_auth(
            &app,
            axum::http::Method::DELETE,
            "/api/permissions/some-id",
            &token,
            None,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}
