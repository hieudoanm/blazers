use super::*;

#[tokio::test]
async fn test_file_token_and_public_bucket() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // protected bucket + file
    svr.post_auth(
        "/api/buckets",
        &admin,
        json!({"name": "private", "is_public": false}),
    )
    .await;
    let r = svr
        .post_multipart(
            "/api/buckets/private/files",
            &admin,
            "a.txt",
            b"secret",
            "text/plain",
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let file_id = r.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // unauthenticated download denied
    let r = svr
        .get(&format!("/api/buckets/private/files/{file_id}"))
        .await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    // file token works without auth
    let r = svr
        .post_auth(
            &format!("/api/buckets/private/files/{file_id}/token"),
            &admin,
            json!({}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let ftok = r.json::<Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();
    let r = svr
        .get(&format!(
            "/api/buckets/private/files/{file_id}?token={ftok}"
        ))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.bytes().await.unwrap().as_ref(), b"secret");

    // public bucket: no auth needed
    svr.post_auth(
        "/api/buckets",
        &admin,
        json!({"name": "public", "is_public": true}),
    )
    .await;
    let r = svr
        .post_multipart(
            "/api/buckets/public/files",
            &admin,
            "b.txt",
            b"open",
            "text/plain",
        )
        .await;
    let pub_id = r.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let r = svr
        .get(&format!("/api/buckets/public/files/{pub_id}"))
        .await;
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_file_token_record_rule() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // a non-admin user (no permissions) can still request file tokens
    svr.post(
        "/api/auth/register",
        json!({"email": "user@test.com", "password": "user123"}),
    )
    .await;
    let r = svr
        .post(
            "/api/auth/login",
            json!({"email": "user@test.com", "password": "user123"}),
        )
        .await;
    let user_token = r.json::<Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // collection whose view_rule denies everyone except admins
    svr.post_auth(
        "/api/collections",
        &admin,
        json!({"name": "docs", "schema": "{}"}),
    )
    .await;
    svr.patch_auth(
        "/api/collections/docs",
        &admin,
        json!({"view_rule": "false"}),
    )
    .await;
    let r = svr
        .post_auth(
            "/api/collections/docs/records",
            &admin,
            json!({"data": {"title": "secret"}}),
        )
        .await;
    let rec_id = r.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    svr.post_auth("/api/buckets", &admin, json!({"name": "priv"}))
        .await;
    let r = svr
        .post_multipart(
            "/api/buckets/priv/files",
            &admin,
            "d.txt",
            b"data",
            "text/plain",
        )
        .await;
    let file_id = r.json::<Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // token without record context: any authenticated user
    let r = svr
        .post_auth(
            &format!("/api/buckets/priv/files/{file_id}/token"),
            &user_token,
            json!({}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // token bound to a record the user cannot view → 403
    let r = svr
        .post_auth(
            &format!("/api/buckets/priv/files/{file_id}/token"),
            &user_token,
            json!({"collection": "docs", "record": rec_id}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::FORBIDDEN);

    // admin bypasses the rule → 200
    let r = svr
        .post_auth(
            &format!("/api/buckets/priv/files/{file_id}/token"),
            &admin,
            json!({"collection": "docs", "record": rec_id}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
}
