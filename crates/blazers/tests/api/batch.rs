use super::*;

#[tokio::test]
async fn test_batch_endpoint() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // create a collection + two records + a read, all in one batch
    let r = svr
        .post_auth(
            "/api/batch",
            &admin,
            json!({"requests": [
                {"method": "POST", "url": "/api/collections", "body": {"name": "items"}},
                {"method": "POST", "url": "/api/collections/items/records", "body": {"data": {"n": 1}}},
                {"method": "POST", "url": "/api/collections/items/records", "body": {"data": {"n": 2}}},
                {"method": "GET", "url": "/api/collections/items/records"}
            ]}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let res = r.json::<Value>().await.unwrap();
    assert_eq!(res.as_array().unwrap().len(), 4);
    assert_eq!(res[0]["status"], 200);
    assert_eq!(res[3]["status"], 200);
    assert_eq!(res[3]["body"]["total"], 2);

    // nested batch rejected per-item, bad method rejected
    let r = svr
        .post_auth(
            "/api/batch",
            &admin,
            json!({"requests": [
                {"method": "POST", "url": "/api/batch", "body": {"requests": []}},
                {"method": "FOO", "url": "/api/health"},
                {"method": "GET", "url": "/etc/passwd"}
            ]}),
        )
        .await;
    let res = r.json::<Value>().await.unwrap();
    assert_eq!(res[0]["status"], 400);
    assert_eq!(res[1]["status"], 400);
    assert_eq!(res[2]["status"], 400);

    // empty batch rejected
    let r = svr
        .post_auth("/api/batch", &admin, json!({"requests": []}))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // unauthenticated rejected
    let r = svr.post("/api/batch", json!({"requests": []})).await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_batch_query_param() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    // ?batch=1 on any /api route accepts a bare JSON array (PocketBase style)
    let r = svr
        .post_auth(
            "/api/collections?batch=1",
            &admin,
            json!([
                {"method": "POST", "url": "/api/collections", "body": {"name": "qp"}},
                {"method": "GET", "url": "/api/collections/qp"}
            ]),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let res = r.json::<Value>().await.unwrap();
    assert_eq!(res.as_array().unwrap().len(), 2);
    assert_eq!(res[0]["status"], 200);
    assert_eq!(res[1]["status"], 200);
    assert_eq!(res[1]["body"]["name"], "qp");

    // the {"requests": [...]} envelope also works with ?batch=1
    let r = svr
        .post_auth(
            "/api/health?batch=1",
            &admin,
            json!({"requests": [{"method": "GET", "url": "/api/health"}]}),
        )
        .await;
    let res = r.json::<Value>().await.unwrap();
    assert_eq!(res[0]["status"], 200);

    // non-array body is a 400, and /api/batch itself is excluded
    let r = svr
        .post_auth("/api/health?batch=1", &admin, json!({"nope": true}))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
