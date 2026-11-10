use super::*;

#[test]
fn export_returns_real_data_and_import_applies_it_to_another_database() {
    let (_dir, db) = test_db();
    let _collection = single(
        call_tool(
            26,
            "blazers_collections_create",
            json!({ "name": "shipped" }),
        ),
        db.clone(),
    );
    let _record = single(
        call_tool(
            27,
            "blazers_records_create",
            json!({
                "collection": "shipped", "id": "r1", "data": { "ok": true }
            }),
        ),
        db.clone(),
    );

    let exported = single(
        call_tool(28, "blazers_export", json!({ "format": "json" })),
        db.clone(),
    );
    let payload = tool_text(&exported);
    assert!(
        payload.contains("shipped"),
        "export is missing the collection"
    );

    // A second database receives the payload, so a passing test cannot be a
    // no-op handler.
    let (_other_dir, other_db) = test_db();
    let imported = single(
        call_tool(
            29,
            "blazers_import",
            json!({ "format": "json", "data": payload }),
        ),
        other_db.clone(),
    );
    let summary = tool_text(&imported);
    assert!(
        summary.contains("\"created_collections\": 1"),
        "import summary: {summary}"
    );

    let listed = single(
        call_tool(30, "blazers_collections_list", json!({})),
        other_db.clone(),
    );
    assert!(tool_text(&listed).contains("shipped"));
    let records = single(
        call_tool(
            31,
            "blazers_records_list",
            json!({ "collection": "shipped" }),
        ),
        other_db,
    );
    assert!(
        tool_text(&records).contains("r1"),
        "imported record is missing"
    );
}

#[test]
fn export_import_round_trip_carries_buckets_and_files() {
    // Export advertises buckets and files, so an import that silently dropped
    // them would turn a round trip into data loss. The destination is a second
    // database, checked row by row rather than through the import summary.
    let (dir, db) = test_db();
    let conn = Connection::open(dir.path().join("data.db")).expect("open");
    db::insert_bucket(&conn, "assets", true).expect("insert bucket");
    db::insert_file(&conn, "assets", "f1", "logo.png", "image/png", 42).expect("insert file");

    let exported = single(
        call_tool(28, "blazers_export", json!({ "format": "json" })),
        db,
    );
    let payload = tool_text(&exported);
    assert!(payload.contains("assets"), "export is missing the bucket");

    let (_other_dir, other_db) = test_db();
    let imported = single(
        call_tool(
            29,
            "blazers_import",
            json!({ "format": "json", "data": payload }),
        ),
        other_db.clone(),
    );
    let summary = tool_text(&imported);
    assert!(
        summary.contains("\"created_buckets\": 1"),
        "import summary: {summary}"
    );
    assert!(
        summary.contains("\"created_files\": 1"),
        "import summary: {summary}"
    );

    let other = other_db.lock().expect("lock");
    let file_count: i64 = other
        .query_row("SELECT COUNT(*) FROM _files", [], |row| row.get(0))
        .expect("count files");
    assert_eq!(file_count, 1, "the file row did not survive the round trip");
    let is_public: i64 = other
        .query_row(
            "SELECT is_public FROM _buckets WHERE name = 'assets'",
            [],
            |row| row.get(0),
        )
        .expect("bucket is_public");
    assert_eq!(is_public, 1, "the bucket did not keep its visibility");
}

#[test]
fn export_rejects_a_format_this_build_cannot_produce() {
    let (_dir, db) = test_db();
    let reply = single(
        call_tool(32, "blazers_export", json!({ "format": "csv" })),
        db,
    );
    assert!(error_text(&reply).contains("only the json format"));
}
