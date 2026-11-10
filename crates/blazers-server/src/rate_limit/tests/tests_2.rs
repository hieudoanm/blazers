use super::*;

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
