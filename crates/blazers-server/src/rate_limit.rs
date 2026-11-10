use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::handlers::AppState;

pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    capacity: f64,
    refill_rate: f64,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            capacity,
            refill_rate,
        }
    }

    pub fn check(&self, ip: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();
        let bucket = buckets.entry(ip.to_string()).or_insert(TokenBucket {
            tokens: self.capacity,
            last_refill: now,
        });
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_rate).min(self.capacity);
        bucket.last_refill = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn cleanup(&self) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.retain(|_, b| b.tokens < self.capacity);
    }
}

pub async fn rate_limit_middleware(
    State(state): State<std::sync::Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            request
                .extensions()
                .get::<axum::extract::connect_info::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
                .unwrap_or_else(|| "127.0.0.1".to_string())
        });

    if !state.rate_limiter.check(&ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "rate limit exceeded"})),
        )
            .into_response();
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn new_creates_rate_limiter() {
        let rl = RateLimiter::new(10.0, 1.0);
        assert_eq!(rl.capacity, 10.0);
        assert_eq!(rl.refill_rate, 1.0);
        assert!(rl.buckets.lock().unwrap().is_empty());
    }

    #[test]
    fn first_request_to_new_ip_is_allowed() {
        let rl = RateLimiter::new(1.0, 0.0);
        assert!(rl.check("192.168.1.1"));
    }

    #[test]
    fn rapid_requests_exhaust_capacity() {
        let rl = RateLimiter::new(3.0, 0.0);
        assert!(rl.check("10.0.0.1"));
        assert!(rl.check("10.0.0.1"));
        assert!(rl.check("10.0.0.1"));
        assert!(!rl.check("10.0.0.1"));
    }

    #[test]
    fn different_ips_have_independent_buckets() {
        let rl = RateLimiter::new(1.0, 0.0);
        assert!(rl.check("ip-a"));
        assert!(!rl.check("ip-a"));
        assert!(rl.check("ip-b"));
    }

    #[test]
    fn cleanup_on_empty_state_is_safe() {
        let rl = RateLimiter::new(10.0, 1.0);
        rl.cleanup();
        assert!(rl.buckets.lock().unwrap().is_empty());
    }

    #[test]
    fn cleanup_removes_full_buckets() {
        let rl = RateLimiter::new(10.0, 0.0);
        {
            let mut buckets = rl.buckets.lock().unwrap();
            buckets.insert(
                "full".into(),
                TokenBucket {
                    tokens: 10.0,
                    last_refill: Instant::now(),
                },
            );
            buckets.insert(
                "active".into(),
                TokenBucket {
                    tokens: 3.0,
                    last_refill: Instant::now(),
                },
            );
        }
        rl.cleanup();
        let buckets = rl.buckets.lock().unwrap();
        assert_eq!(buckets.len(), 1);
        assert!(buckets.contains_key("active"));
        assert!(!buckets.contains_key("full"));
    }

    #[test]
    fn cleanup_preserves_partial_buckets() {
        let rl = RateLimiter::new(10.0, 0.0);
        {
            let mut buckets = rl.buckets.lock().unwrap();
            buckets.insert(
                "low".into(),
                TokenBucket {
                    tokens: 0.5,
                    last_refill: Instant::now(),
                },
            );
            buckets.insert(
                "mid".into(),
                TokenBucket {
                    tokens: 5.0,
                    last_refill: Instant::now(),
                },
            );
        }
        rl.cleanup();
        let buckets = rl.buckets.lock().unwrap();
        assert_eq!(buckets.len(), 2);
    }

    #[test]
    fn cleanup_removes_buckets_refilled_to_capacity() {
        let rl = RateLimiter::new(5.0, 0.0);
        {
            let mut buckets = rl.buckets.lock().unwrap();
            buckets.insert(
                "ip1".into(),
                TokenBucket {
                    tokens: 5.0,
                    last_refill: Instant::now(),
                },
            );
            buckets.insert(
                "ip2".into(),
                TokenBucket {
                    tokens: 5.0,
                    last_refill: Instant::now(),
                },
            );
        }
        rl.cleanup();
        assert!(rl.buckets.lock().unwrap().is_empty());
    }

    // --- rate_limit_middleware tests ---

    async fn make_rate_limit_app(
        capacity: f64,
        refill_rate: f64,
    ) -> (
        axum::Router,
        std::sync::Arc<crate::handlers::AppState>,
        PathBuf,
    ) {
        use axum::{Router, middleware, routing::get};
        use blazers_core::db::ConnectionManager;
        use deadpool::managed::Pool;

        let tmp_dir = std::env::temp_dir().join(format!("blazers-rate-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let db_path = tmp_dir.join("test.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        blazers_core::db::migrate_db(&conn).unwrap();
        drop(conn);
        let pool: Pool<ConnectionManager> = Pool::builder(ConnectionManager { path: db_path })
            .build()
            .unwrap();
        let rate_limiter = std::sync::Arc::new(RateLimiter::new(capacity, refill_rate));
        let state = std::sync::Arc::new(crate::handlers::AppState {
            db: pool,
            storage_dir: tmp_dir.clone(),
            http_client: reqwest::Client::new(),
            secrets_key: vec![1, 2, 3, 4],
            ws_hub: crate::websocket::new_ws_hub(),
            cache: std::sync::Arc::new(crate::cache::CacheStore::new()),
            sse_hub: crate::notification::new_sse_hub(),
            log_hub: crate::log::new_log_hub(),
            pubsub_hub: crate::pubsub::new_pubsub_hub(),
            rate_limiter,
        });

        let app = Router::new()
            .route("/", get(|| async { "OK" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                rate_limit_middleware,
            ))
            .with_state(state.clone());

        (app, state, tmp_dir)
    }

    #[tokio::test]
    async fn middleware_allows_first_request() {
        let (app, _state, _tmp) = make_rate_limit_app(1.0, 0.0).await;
        use axum::{body::Body, http::Request, http::StatusCode};
        use tower::ServiceExt;

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn middleware_blocks_when_exhausted() {
        let (app, _state, _tmp) = make_rate_limit_app(1.0, 0.0).await;
        use axum::{body::Body, http::Request, http::StatusCode};
        use tower::ServiceExt;

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn middleware_returns_rate_limit_json_error() {
        let (app, _state, _tmp) = make_rate_limit_app(1.0, 0.0).await;
        use axum::{body::Body, http::Request, http::StatusCode};
        use tower::ServiceExt;

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let _ = app.clone().oneshot(req).await.unwrap();

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "rate limit exceeded");
    }

    #[tokio::test]
    async fn middleware_uses_x_forwarded_for() {
        let (app, _state, _tmp) = make_rate_limit_app(1.0, 0.0).await;
        use axum::{body::Body, http::Request, http::StatusCode};
        use tower::ServiceExt;

        let req = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let req = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        let req = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.2")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn middleware_uses_first_ip_from_forwarded_list() {
        let (app, _state, _tmp) = make_rate_limit_app(1.0, 0.0).await;
        use axum::{body::Body, http::Request, http::StatusCode};
        use tower::ServiceExt;

        let req = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.1, 10.0.0.2")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let req = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.1, 10.0.0.3")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        let req = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "10.0.0.2, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
