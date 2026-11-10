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
mod tests;
