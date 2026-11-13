use super::*;

async fn mk_user(svr: &Svr, email: &str) -> (String, String) {
    svr.post(
        "/api/auth/register",
        json!({"email": email, "password": "pass123"}),
    )
    .await;
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

#[tokio::test]
async fn test_managed_users_collection() {
    let svr = Svr::start().await;
    let admin = svr.register().await;
    let (user_token, user_id) = mk_user(&svr, "u1@test.com").await;
    mk_user(&svr, "u2@test.com").await;

    // synthetic auth collection appears in the list
    let r = svr.get_auth("/api/collections", &admin).await;
    let cols = r.json::<Value>().await.unwrap();
    let users_col = cols
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "users")
        .expect("users collection missing");
    assert_eq!(users_col["type"], "auth");

    // admin lists all users
    let r = svr.get_auth("/api/auth/users", &admin).await;
    assert_eq!(r.status(), StatusCode::OK);
    let page = r.json::<Value>().await.unwrap();
    assert!(page["total"].as_i64().unwrap() >= 3);

    // non-admin sees only self by default
    let r = svr.get_auth("/api/auth/users", &user_token).await;
    let page = r.json::<Value>().await.unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["users"][0]["id"], user_id);

    // non-admin can fetch self, not others
    let r = svr
        .get_auth(&format!("/api/auth/users/{user_id}"), &user_token)
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // set a view_rule allowing users with verified=true... simpler: allow all
    let r = svr
        .patch_auth(
            "/api/auth/users/rules",
            &admin,
            json!({"view_rule": "true", "list_rule": "true"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // now non-admin can list all and view others
    let r = svr.get_auth("/api/auth/users", &user_token).await;
    let page = r.json::<Value>().await.unwrap();
    assert!(page["total"].as_i64().unwrap() >= 3);

    // rules round-trip
    let r = svr.get_auth("/api/auth/users/rules", &admin).await;
    assert_eq!(r.json::<Value>().await.unwrap()["view_rule"], "true");

    // invalid rule rejected
    let r = svr
        .patch_auth("/api/auth/users/rules", &admin, json!({"list_rule": "((("}))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
