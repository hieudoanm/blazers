use super::*;

async fn register_user(svr: &Svr, email: &str) -> (String, String) {
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": email, "password": "user123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": email, "password": "user123"}),
        )
        .await;
    let v = r.json::<Value>().await.unwrap();
    (
        v["token"].as_str().unwrap().to_string(),
        v["user"]["id"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn test_otp_login() {
    let svr = Svr::start().await;
    let _admin = svr.register().await;

    // unknown email still returns 200 (no enumeration) but no dev_otp
    let r = svr
        .post("/api/auth/request-otp", json!({"email": "ghost@test.com"}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(r.json::<Value>().await.unwrap()["dev_otp"].is_null());

    let r = svr
        .post("/api/auth/request-otp", json!({"email": "admin@test.com"}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let otp = r.json::<Value>().await.unwrap()["dev_otp"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(otp.len(), 6);

    // wrong code rejected
    let r = svr
        .post(
            "/api/auth/auth-with-otp",
            json!({"email": "admin@test.com", "otp": "000000"}),
        )
        .await;
    if otp != "000000" {
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    }

    // correct code → JWT that works on /me
    let r = svr
        .post(
            "/api/auth/auth-with-otp",
            json!({"email": "admin@test.com", "otp": otp}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let v = r.json::<Value>().await.unwrap();
    let token = v["token"].as_str().unwrap().to_string();
    let r = svr.get_auth("/api/auth/me", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.json::<Value>().await.unwrap()["email"], "admin@test.com");

    // code is single-use
    let r = svr
        .post(
            "/api/auth/auth-with-otp",
            json!({"email": "admin@test.com", "otp": otp}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_impersonate() {
    let svr = Svr::start().await;
    let admin = svr.register().await;
    let (user_token, user_id) = register_user(&svr, "target@test.com").await;

    // non-admin cannot impersonate
    let r = svr
        .post_auth(
            &format!("/api/auth/users/{user_id}/impersonate"),
            &user_token,
            json!({}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::FORBIDDEN);

    // admin gets a user-scoped JWT for the target
    let r = svr
        .post_auth(
            &format!("/api/auth/users/{user_id}/impersonate"),
            &admin,
            json!({}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let v = r.json::<Value>().await.unwrap();
    assert_eq!(v["user"]["email"], "target@test.com");
    let imp = v["token"].as_str().unwrap().to_string();

    let r = svr.get_auth("/api/auth/me", &imp).await;
    let me = r.json::<Value>().await.unwrap();
    assert_eq!(me["email"], "target@test.com");
    assert_eq!(me["id"], user_id);

    // unknown user → 404
    let r = svr
        .post_auth("/api/auth/users/nope/impersonate", &admin, json!({}))
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_oauth_endpoints() {
    let svr = Svr::start().await;

    // unknown provider → 404
    let r = svr.get("/api/auth/oauth/gitbucket").await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    // known providers without client config → 400
    for p in ["google", "github", "gitlab", "discord"] {
        let r = svr.get(&format!("/api/auth/oauth/{p}")).await;
        assert_eq!(r.status(), StatusCode::BAD_REQUEST, "provider {p}");
    }

    // callback rejects bad state before touching provider APIs
    let r = svr
        .get("/api/auth/oauth/google/callback?code=abc&state=bad")
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // a real state token round-trips (issued by oauth_start would need
    // config; here we only assert tampered states are rejected)
    let r = svr.get("/api/auth/oauth/github/callback?state=bad").await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
