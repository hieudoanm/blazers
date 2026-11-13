use super::*;

async fn mk_user(svr: &Svr, email: &str) -> (String, String) {
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": email, "password": "pass123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK, "register {email}");
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": email, "password": "pass123"}),
        )
        .await;
    let body = r.json::<Value>().await.unwrap();
    (
        body["token"].as_str().unwrap().to_string(),
        body["user"]["id"].as_str().unwrap().to_string(),
    )
}

mod auth;
mod files;
mod records;
mod settings;
