use super::*;
use totp_rs::{Algorithm, Secret, TOTP};

fn current_code(secret_b32: &str) -> String {
    let bytes = Secret::Encoded(secret_b32.to_string()).to_bytes().unwrap();
    TOTP::new(Algorithm::SHA1, 6, 1, 30, bytes)
        .unwrap()
        .generate_current()
        .unwrap()
}

#[tokio::test]
async fn test_refresh_token() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // valid token → fresh JWT
    let r = svr.post_auth("/api/auth/refresh", &admin, json!({})).await;
    assert_eq!(r.status(), StatusCode::OK);
    let v = r.json::<Value>().await.unwrap();
    let new_token = v["token"].as_str().unwrap().to_string();
    // same-second JWTs are deterministic, so just verify it works
    let r = svr.get_auth("/api/auth/me", &new_token).await;
    assert_eq!(r.status(), StatusCode::OK);

    // no token → 401
    let r = svr.post("/api/auth/refresh", json!({})).await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_mfa_flow() {
    let svr = Svr::start().await;
    let _admin = svr.register().await;
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": "mfa@test.com", "password": "mfa12345"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "mfa@test.com", "password": "mfa12345"}),
        )
        .await;
    let token = r.json::<Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // enable → secret; confirm with a valid code
    let r = svr
        .post_auth("/api/auth/mfa/enable", &token, json!({}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let v = r.json::<Value>().await.unwrap();
    let secret = v["secret"].as_str().unwrap().to_string();
    assert!(
        v["otpauth_url"]
            .as_str()
            .unwrap()
            .contains("otpauth://totp/")
    );

    let code = current_code(&secret);
    let r = svr
        .post_auth("/api/auth/mfa/confirm", &token, json!({"code": code}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // login now requires MFA
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "mfa@test.com", "password": "mfa12345"}),
        )
        .await;
    let v = r.json::<Value>().await.unwrap();
    assert_eq!(v["mfa_required"], true);
    assert!(v.get("token").is_none());

    // wrong code rejected, right code → JWT
    let r = svr
        .post(
            "/api/auth/mfa/login",
            json!({"email": "mfa@test.com", "password": "mfa12345", "code": "000000"}),
        )
        .await;
    if code != "000000" {
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    }
    let r = svr
        .post(
            "/api/auth/mfa/login",
            json!({"email": "mfa@test.com", "password": "mfa12345", "code": code}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let mfa_token = r.json::<Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let r = svr.get_auth("/api/auth/me", &mfa_token).await;
    assert_eq!(r.json::<Value>().await.unwrap()["email"], "mfa@test.com");

    // disable with a fresh code
    let r = svr
        .post_auth(
            "/api/auth/mfa/disable",
            &mfa_token,
            json!({"code": current_code(&secret)}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "mfa@test.com", "password": "mfa12345"}),
        )
        .await;
    assert!(r.json::<Value>().await.unwrap().get("token").is_some());
}
