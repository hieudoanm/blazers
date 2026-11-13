use super::*;

#[tokio::test]
async fn test_anonymous_sign_in_and_link() {
    let svr = Svr::start().await;

    // anonymous sign-in returns a session immediately
    let r = svr.post("/api/auth/anonymous", json!({})).await;
    assert_eq!(r.status(), StatusCode::OK);
    let v = r.json::<Value>().await.unwrap();
    let token = v["token"].as_str().unwrap().to_string();
    assert_eq!(v["user"]["data"]["is_anonymous"], true);
    let anon_email = v["user"]["email"].as_str().unwrap().to_string();
    assert!(anon_email.starts_with("anon-"));

    // /me works with the anonymous token
    let r = svr.get_auth("/api/auth/me", &token).await;
    assert_eq!(r.status(), StatusCode::OK);

    // link: set a real email + password, is_anonymous flips to false
    let r = svr
        .patch_auth(
            "/api/auth/me",
            &token,
            json!({"email": "linked@test.com", "password": "linked123", "data": {"is_anonymous": true}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let v = r.json::<Value>().await.unwrap();
    assert_eq!(v["email"], "linked@test.com");
    assert_eq!(v["data"]["is_anonymous"], false);

    // can now log in with email + password
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "linked@test.com", "password": "linked123"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(r.json::<Value>().await.unwrap().get("token").is_some());

    // duplicate email is rejected
    let r = svr.post("/api/auth/anonymous", json!({})).await;
    let token2 = r.json::<Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let r = svr
        .patch_auth("/api/auth/me", &token2, json!({"email": "linked@test.com"}))
        .await;
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_typed_collection_schema() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // invalid schema JSON → 400
    let r = svr
        .post_auth(
            "/api/collections",
            &admin,
            json!({"name": "bad_schema", "schema": "not-json"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // unknown type → 400
    let r = svr
        .post_auth(
            "/api/collections",
            &admin,
            json!({"name": "bad_type", "schema": r#"{"age": "float"}"#}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // valid typed schema
    let r = svr
        .post_auth(
            "/api/collections",
            &admin,
            json!({"name": "articles", "schema": r#"{"title": "string", "views": "integer", "published": "boolean", "tags?": "array"}"#}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // missing required field → 400
    let r = svr
        .post_auth(
            "/api/collections/articles/records",
            &admin,
            json!({"data": {"views": 3}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // wrong type → 400
    let r = svr
        .post_auth(
            "/api/collections/articles/records",
            &admin,
            json!({"data": {"title": "hi", "views": "not-a-number"}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // valid record → 200
    let r = svr
        .post_auth(
            "/api/collections/articles/records",
            &admin,
            json!({"data": {"title": "hello", "views": 10, "published": true}}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_functions_crud_and_invoke() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // non-admin cannot manage functions — register a second user
    let r = svr
        .post(
            "/api/auth/register",
            json!({"email": "pleb@test.com", "password": "pleb1234"}),
        )
        .await;
    let pleb_token = r.json::<Value>().await.unwrap();
    let _ = pleb_token;
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "pleb@test.com", "password": "pleb1234"}),
        )
        .await;
    let pleb = r.json::<Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // create function (admin)
    let r = svr
        .post_auth(
            "/api/functions",
            &admin,
            json!({
                "name": "double",
                "code": "function handler(ctx) { return { value: ctx.input.n * 2 }; }"
            }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // non-admin list → 403
    let r = svr.get_auth("/api/functions", &pleb).await;
    assert_eq!(r.status(), StatusCode::FORBIDDEN);

    // invoke without auth → works, auth is null
    let r = svr
        .post("/api/functions/double", json!({"input": {"n": 21}}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let v = r.json::<Value>().await.unwrap();
    assert_eq!(v["result"]["value"], 42);

    // invoke with auth → auth context visible
    let r = svr
        .post_auth("/api/functions/double", &admin, json!({"input": {"n": 1}}))
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // function returning auth info
    svr.post_auth(
        "/api/functions",
        &admin,
        json!({
            "name": "whoami",
            "code": "function handler(ctx) { return { email: ctx.auth ? ctx.auth.email : null }; }"
        }),
    )
    .await;
    let r = svr
        .post_auth("/api/functions/whoami", &admin, json!({"input": {}}))
        .await;
    let v = r.json::<Value>().await.unwrap();
    assert_eq!(v["result"]["email"], "admin@test.com");

    // JS error → 400
    let svr2 = Svr::start().await;
    let admin2 = svr2.register().await;
    svr2.post_auth(
        "/api/functions",
        &admin2,
        json!({"name": "bad", "code": "function handler() { throw new Error('boom'); }"}),
    )
    .await;
    let r = svr2.post("/api/functions/bad", json!({"input": {}})).await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // missing function → 404
    let r = svr.post("/api/functions/nope", json!({"input": {}})).await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    // cleanup: delete function (admin)
    let r = svr.delete_auth("/api/functions/double", &admin).await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    let r = svr
        .post("/api/functions/double", json!({"input": {}}))
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
