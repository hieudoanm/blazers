use super::*;

#[tokio::test]
async fn test_record_filter_sort_expand() {
    let svr = Svr::start().await;
    let admin = svr.register().await;

    svr.post_auth("/api/collections", &admin, json!({"name": "posts"}))
        .await;
    svr.post_auth("/api/collections", &admin, json!({"name": "authors"}))
        .await;
    let author = svr
        .post_auth(
            "/api/collections/authors/records",
            &admin,
            json!({"data": {"name": "Alice"}}),
        )
        .await
        .json::<Value>()
        .await
        .unwrap();
    let author_id = author["id"].as_str().unwrap();

    for (title, views, published) in [
        ("rust guide", 100, true),
        ("db internals", 5, true),
        ("draft notes", 50, false),
    ] {
        svr.post_auth(
            "/api/collections/posts/records",
            &admin,
            json!({"data": {"title": title, "views": views, "published": published, "author": format!("authors:{author_id}")}}),
        )
        .await;
    }

    // filter
    let r = svr
        .get_auth(
            "/api/collections/posts/records?filter=views%20%3E%2040",
            &admin,
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let page = r.json::<Value>().await.unwrap();
    assert_eq!(page["total"], 2);

    // compound filter
    let r = svr
        .get_auth(
            "/api/collections/posts/records?filter=published%20%3D%20true%20%26%26%20views%20%3C%2060",
            &admin,
        )
        .await;
    let page = r.json::<Value>().await.unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["records"][0]["data"]["title"], "db internals");

    // sort
    let r = svr
        .get_auth("/api/collections/posts/records?sort=-views", &admin)
        .await;
    let page = r.json::<Value>().await.unwrap();
    assert_eq!(page["records"][0]["data"]["title"], "rust guide");

    // expand
    let r = svr
        .get_auth("/api/collections/posts/records?expand=author", &admin)
        .await;
    let page = r.json::<Value>().await.unwrap();
    assert_eq!(
        page["records"][0]["data"]["expand_author"]["data"]["name"],
        "Alice"
    );
}

#[tokio::test]
async fn test_record_rules() {
    let svr = Svr::start().await;
    let admin = svr.register().await;
    let (viewer_token, viewer_id) = mk_user(&svr, "viewer@test.com").await;

    // viewer can list the collection but only sees own records
    svr.post_auth("/api/collections", &admin, json!({"name": "notes"}))
        .await;
    // grant viewer role so RBAC passes; rules then restrict rows
    let r = svr
        .post_auth(
            "/api/permissions",
            &admin,
            json!({"user_id": viewer_id, "collection": "notes", "role": "viewer"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::CREATED);

    // set list/view rules: owner = @request.auth.id
    let r = svr
        .patch_auth(
            "/api/collections/notes",
            &admin,
            json!({"list_rule": "owner = @request.auth.id", "view_rule": "owner = @request.auth.id"}),
        )
        .await;
    assert_eq!(r.status(), StatusCode::OK);

    // admin record owned by admin
    let admin_id = svr
        .get_auth("/api/auth/me", &admin)
        .await
        .json::<Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    svr.post_auth(
        "/api/collections/notes/records",
        &admin,
        json!({"data": {"title": "admin note", "owner": admin_id}}),
    )
    .await;
    let r = svr
        .get_auth("/api/collections/notes/records", &viewer_token)
        .await;
    assert_eq!(r.status(), StatusCode::OK);
    let page = r.json::<Value>().await.unwrap();
    // viewer sees zero records (admin's record filtered out by rule)
    assert_eq!(page["total"], 0);

    // admin still sees all
    let r = svr.get_auth("/api/collections/notes/records", &admin).await;
    let page = r.json::<Value>().await.unwrap();
    assert_eq!(page["total"], 1);
}
