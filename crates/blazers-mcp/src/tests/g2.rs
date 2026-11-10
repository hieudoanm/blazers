use super::*;

#[test]
fn records_round_trip_through_the_real_database() {
    let (_dir, db) = test_db();
    let _collection = single(
        call_tool(13, "blazers_collections_create", json!({ "name": "posts" })),
        db.clone(),
    );

    let created = single(
        call_tool(
            14,
            "blazers_records_create",
            json!({
                "collection": "posts", "id": "post-1", "data": { "title": "Hello" }
            }),
        ),
        db.clone(),
    );
    assert!(tool_text(&created).contains("Hello"));

    let fetched = single(
        call_tool(
            15,
            "blazers_records_get",
            json!({ "collection": "posts", "id": "post-1" }),
        ),
        db.clone(),
    );
    assert!(tool_text(&fetched).contains("Hello"));

    let updated = single(
        call_tool(
            16,
            "blazers_records_update",
            json!({
                "collection": "posts", "id": "post-1", "data": { "title": "Updated" }
            }),
        ),
        db.clone(),
    );
    assert!(tool_text(&updated).contains("Updated"));

    let listed = single(
        call_tool(17, "blazers_records_list", json!({ "collection": "posts" })),
        db.clone(),
    );
    assert!(
        tool_text(&listed).contains("Updated"),
        "update was not persisted"
    );

    let deleted = single(
        call_tool(
            18,
            "blazers_records_delete",
            json!({ "collection": "posts", "id": "post-1" }),
        ),
        db.clone(),
    );
    assert!(tool_text(&deleted).contains("\"deleted\": true"));

    let missing = single(
        call_tool(
            19,
            "blazers_records_get",
            json!({ "collection": "posts", "id": "post-1" }),
        ),
        db,
    );
    assert!(error_text(&missing).contains("not found"));
}

#[test]
fn records_create_generates_an_id_when_none_is_given() {
    let (_dir, db) = test_db();
    let _collection = single(
        call_tool(20, "blazers_collections_create", json!({ "name": "auto" })),
        db.clone(),
    );
    let created = single(
        call_tool(
            21,
            "blazers_records_create",
            json!({ "collection": "auto", "data": { "n": 1 } }),
        ),
        db,
    );
    assert!(
        tool_text(&created).contains("\"id\""),
        "no id was generated"
    );
}

#[test]
fn records_list_rejects_a_per_page_above_the_cap() {
    let (_dir, db) = test_db();
    let _collection = single(
        call_tool(
            22,
            "blazers_collections_create",
            json!({ "name": "capped" }),
        ),
        db.clone(),
    );
    let reply = single(
        call_tool(
            23,
            "blazers_records_list",
            json!({ "collection": "capped", "per_page": 100_000 }),
        ),
        db,
    );
    assert!(error_text(&reply).contains("per_page must be between"));
}

#[test]
fn records_list_rejects_a_page_below_one() {
    let (_dir, db) = test_db();
    let _collection = single(
        call_tool(24, "blazers_collections_create", json!({ "name": "paged" })),
        db.clone(),
    );
    let reply = single(
        call_tool(
            25,
            "blazers_records_list",
            json!({ "collection": "paged", "page": 0 }),
        ),
        db,
    );
    assert!(error_text(&reply).contains("page must be 1 or greater"));
}
