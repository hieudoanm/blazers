use super::*;

#[tokio::test]
async fn test_auth_me_and_password_reset() {
    let svr = Svr::start().await;
    let (token, user_id) = mk_user(&svr, "me@test.com").await;

    // me
    let r = svr.get_auth("/api/auth/me", &token).await;
    assert_eq!(r.status(), StatusCode::OK);
    let me = r.json::<Value>().await.unwrap();
    assert_eq!(me["id"], user_id);
    assert_eq!(me["email"], "me@test.com");
    assert_eq!(me["verified"], false);

    // update custom data
    let r = svr
        .patch_auth(
            "/api/auth/me",
            &token,
            json!({"data": {"name": "Me User", "age": 30}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let me = r.json::<Value>().await.unwrap();
    assert_eq!(me["data"]["name"], "Me User");

    // forgot password (no SMTP → dev_token returned)
    let r = svr
        .post("/api/auth/forgot-password", json!({"email": "me@test.com"}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    let reset_token = body["dev_token"].as_str().unwrap().to_string();
    assert!(!reset_token.is_empty());

    // reset password
    let r = svr
        .post(
            "/api/auth/reset-password",
            json!({"token": reset_token, "password": "newpass99"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // old password fails, new works
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "me@test.com", "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "me@test.com", "password": "newpass99"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // token reuse rejected
    let r = svr
        .post(
            "/api/auth/reset-password",
            json!({"token": reset_token, "password": "another1"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_email_verification() {
    let svr = Svr::start().await;
    let (token, _) = mk_user(&svr, "verify@test.com").await;

    // request verification → dev_token (no SMTP)
    let r = svr
        .post_auth("/api/auth/verify-email", &token, json!({}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = r.json::<Value>().await.unwrap();
    let verify_token = body["dev_token"].as_str().unwrap().to_string();

    // confirm
    let r = svr
        .post(
            "/api/auth/verify-email/confirm",
            json!({"token": verify_token}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // me shows verified
    let r = svr.get_auth("/api/auth/me", &token).await;
    assert_eq!(r.json::<Value>().await.unwrap()["verified"], true);
}
