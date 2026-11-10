use super::*;

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
