use super::*;

#[tokio::test]
async fn test_health_and_auth() {
    let svr = Svr::start().await;

    // Health
    let r = svr.get("/api/health").await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["status"], "ok");

    // Register
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": "user@test.com", "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    assert_eq!(body["email"], "user@test.com");
    assert!(!body["id"].as_str().unwrap().is_empty());

    // Duplicate register
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": "user@test.com", "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::CONFLICT);

    // Login
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "user@test.com", "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    assert!(!body["token"].as_str().unwrap().is_empty());
    assert_eq!(body["user"]["email"], "user@test.com");

    // Bad password
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "user@test.com", "password": "wrong"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    // Missing fields
    let r = svr.post("/api/auth/register", json!({})).await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Auth guard — /api/collections without token
    let r = svr.get("/api/collections").await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    // Invalid token
    let r = svr.get_auth("/api/collections", "invalid-token").await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
}
