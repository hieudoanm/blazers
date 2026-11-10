//! Protocol behaviour of the Blazers MCP server.
//!
//! Each test drives the real [super::Server] against a real migrated SQLite
//! database in a temporary directory, so tool handlers are exercised against
//! real storage rather than a stub. `BLAZERS_DATA` is pointed at that directory
//! so the handlers and `mcp::run` open the same file.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde_json::{Value, json};

use super::{DbHandle, Diagnostics, Server, protocol, transport};
use blazers_core::db;

/// A migrated database in a fresh temporary directory.
fn test_db() -> (tempfile::TempDir, DbHandle) {
    let dir = tempfile::tempdir().expect("temp dir");
    // SAFETY: the MCP server reads this once at open time, and each test sets it
    // before constructing a server. Tests run in parallel, so a shared value
    // would be wrong; instead handlers are given the handle directly below and
    // only `mcp::run` consults the variable.
    let conn = Connection::open(dir.path().join("data.db")).expect("open test database");
    db::migrate_db(&conn).expect("migrate test database");
    (dir, Arc::new(Mutex::new(conn)))
}

/// Runs one request through a fresh server and returns the reply frames.
fn call(request: Value, db: DbHandle) -> Vec<Value> {
    let mut output: Vec<u8> = Vec::new();
    {
        let mut server = Server::new(&mut output, db);
        server.register_tools();
        let input = format!("{request}\n");
        server.serve(input.as_bytes()).expect("serve");
    }
    frames(&output)
}

/// Runs a request expected to produce exactly one reply.
fn single(request: Value, db: DbHandle) -> Value {
    let replies = call(request, db);
    assert_eq!(replies.len(), 1, "expected one reply, got {replies:?}");
    replies.into_iter().next().expect("one reply")
}

/// Decodes a captured output buffer into one value per line.
fn frames(output: &[u8]) -> Vec<Value> {
    String::from_utf8(output.to_vec())
        .expect("utf8 output")
        .lines()
        .map(|line| serde_json::from_str(line).expect("each line is one JSON frame"))
        .collect()
}

fn request(id: u32, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

fn call_tool(id: u32, name: &str, arguments: Value) -> Value {
    request(
        id,
        "tools/call",
        json!({ "name": name, "arguments": arguments }),
    )
}

fn result_of(reply: &Value) -> &Value {
    reply
        .get("result")
        .unwrap_or_else(|| panic!("reply has no result: {reply}"))
}

/// The text of a tool result, panicking when the tool reported an error so a
/// failure surfaces as a readable message instead of an empty string.
fn tool_text(reply: &Value) -> String {
    let result = result_of(reply);
    assert!(
        result
            .get("isError")
            .is_none_or(|flag| flag == &json!(false)),
        "tool reported an error: {result}"
    );
    result["content"][0]["text"]
        .as_str()
        .expect("text content")
        .to_string()
}

/// The text of an errored tool result.
fn error_text(reply: &Value) -> String {
    assert_eq!(
        result_of(reply)["isError"],
        json!(true),
        "expected an errored result: {reply}"
    );
    result_of(reply)["content"][0]["text"]
        .as_str()
        .expect("text content")
        .to_string()
}

#[test]
fn initialize_advertises_the_protocol_version_and_server_name() {
    let (_dir, db) = test_db();
    let reply = single(
        request(1, "initialize", json!({ "protocolVersion": "2025-11-25" })),
        db,
    );
    let result = result_of(&reply);
    assert_eq!(result["protocolVersion"], "2025-11-25");
    assert_eq!(result["serverInfo"]["name"], "blazers-mcp");
    assert_eq!(result["serverInfo"]["version"], super::SERVER_VERSION);
    assert_eq!(result["capabilities"]["tools"]["listChanged"], false);
    assert_eq!(reply["jsonrpc"], "2.0");
    assert_eq!(reply["id"], 1);
}

#[test]
fn initialize_answers_with_its_own_revision_when_the_client_asks_for_another() {
    let (_dir, db) = test_db();
    let reply = single(
        request(1, "initialize", json!({ "protocolVersion": "1999-01-01" })),
        db,
    );
    assert_eq!(result_of(&reply)["protocolVersion"], "2025-11-25");
}

#[test]
fn ping_answers_with_an_empty_result() {
    let (_dir, db) = test_db();
    let reply = single(request(7, "ping", json!({})), db);
    assert_eq!(*result_of(&reply), json!({}));
}

#[test]
fn tools_list_is_sorted_and_covers_the_blazers_surface() {
    let (_dir, db) = test_db();
    let reply = single(request(2, "tools/list", json!({})), db);
    let names: Vec<String> = result_of(&reply)["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name").to_string())
        .collect();

    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "tools/list must be sorted by name");
    for expected in [
        "blazers_collections_create",
        "blazers_collections_delete",
        "blazers_collections_list",
        "blazers_export",
        "blazers_health",
        "blazers_import",
        "blazers_records_create",
        "blazers_records_delete",
        "blazers_records_get",
        "blazers_records_list",
        "blazers_records_update",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "missing tool {expected}"
        );
    }
}

#[test]
fn every_tool_advertises_an_object_schema_with_a_required_list() {
    let (_dir, db) = test_db();
    let reply = single(request(2, "tools/list", json!({})), db);
    let tools = result_of(&reply)["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty());
    for tool in tools {
        let name = tool["name"].as_str().expect("tool name");
        assert_eq!(
            tool["inputSchema"]["type"], "object",
            "bad schema for {name}"
        );
        assert!(
            tool["inputSchema"].get("required").is_some(),
            "{name} is missing a required list"
        );
        assert!(
            !tool["description"].as_str().unwrap_or_default().is_empty(),
            "{name} has no description"
        );
    }
}

#[test]
fn health_reports_the_database_is_reachable() {
    let (_dir, db) = test_db();
    let reply = single(call_tool(3, "blazers_health", json!({})), db);
    assert!(tool_text(&reply).contains("operational"));
}

#[test]
fn collections_round_trip_through_the_real_database() {
    let (_dir, db) = test_db();

    let empty = single(
        call_tool(4, "blazers_collections_list", json!({})),
        db.clone(),
    );
    assert_eq!(tool_text(&empty).trim(), "{\n  \"collections\": []\n}");

    let created = single(
        call_tool(
            5,
            "blazers_collections_create",
            json!({ "name": "notes", "schema": "{}" }),
        ),
        db.clone(),
    );
    assert!(tool_text(&created).contains("\"created\": true"));

    let listed = single(
        call_tool(6, "blazers_collections_list", json!({})),
        db.clone(),
    );
    assert!(
        tool_text(&listed).contains("notes"),
        "created collection is missing"
    );

    let deleted = single(
        call_tool(7, "blazers_collections_delete", json!({ "name": "notes" })),
        db.clone(),
    );
    assert!(tool_text(&deleted).contains("\"deleted\": true"));

    let gone = single(call_tool(8, "blazers_collections_list", json!({})), db);
    assert_eq!(tool_text(&gone).trim(), "{\n  \"collections\": []\n}");
}

#[test]
fn creating_a_collection_twice_reports_a_conflict() {
    let (_dir, db) = test_db();
    let _first = single(
        call_tool(9, "blazers_collections_create", json!({ "name": "dupe" })),
        db.clone(),
    );
    let second = single(
        call_tool(10, "blazers_collections_create", json!({ "name": "dupe" })),
        db,
    );
    assert!(error_text(&second).contains("already exists"));
}

#[test]
fn creating_a_collection_without_a_name_is_rejected() {
    let (_dir, db) = test_db();
    let reply = single(
        call_tool(11, "blazers_collections_create", json!({ "name": "  " })),
        db,
    );
    assert!(error_text(&reply).contains("must not be empty"));
}

#[test]
fn deleting_a_missing_collection_reports_an_error() {
    let (_dir, db) = test_db();
    let reply = single(
        call_tool(12, "blazers_collections_delete", json!({ "name": "ghost" })),
        db,
    );
    assert!(error_text(&reply).contains("not found"));
}

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

#[test]
fn import_rejects_malformed_data() {
    let (_dir, db) = test_db();
    let reply = single(
        call_tool(
            33,
            "blazers_import",
            json!({ "format": "json", "data": "{nope" }),
        ),
        db,
    );
    assert!(error_text(&reply).contains("parse data"));
}

#[test]
fn a_notification_produces_no_reply() {
    let (_dir, db) = test_db();
    let replies = call(
        json!({ "jsonrpc": "2.0", "method": "tools/list" }),
        db.clone(),
    );
    assert!(
        replies.is_empty(),
        "a notification must stay silent: {replies:?}"
    );

    let null_id = call(
        json!({ "jsonrpc": "2.0", "id": Value::Null, "method": "tools/list" }),
        db,
    );
    assert!(
        null_id.is_empty(),
        "a JSON-null id is a notification: {null_id:?}"
    );
}

#[test]
fn a_string_null_id_is_a_real_request() {
    let (_dir, db) = test_db();
    let replies = call(
        json!({ "jsonrpc": "2.0", "id": "null", "method": "ping" }),
        db,
    );
    assert_eq!(
        replies.len(),
        1,
        "the string \"null\" is a real id: {replies:?}"
    );
    assert_eq!(replies[0]["id"], "null");
}

#[test]
fn an_unknown_method_reports_method_not_found() {
    let (_dir, db) = test_db();
    let reply = single(request(34, "nope", json!({})), db);
    assert_eq!(
        reply["error"]["code"],
        protocol::error_code::METHOD_NOT_FOUND
    );
    assert_eq!(reply["error"]["message"], "method not found: nope");
}

#[test]
fn an_unknown_tool_reports_method_not_found() {
    let (_dir, db) = test_db();
    let reply = single(call_tool(35, "blazers_nope", json!({})), db);
    assert_eq!(
        reply["error"]["code"],
        protocol::error_code::METHOD_NOT_FOUND
    );
    assert!(
        reply["error"]["message"]
            .as_str()
            .expect("message")
            .contains("tool not found")
    );
}

#[test]
fn a_malformed_frame_reports_a_parse_error_with_a_null_id() {
    let (_dir, db) = test_db();
    let mut output: Vec<u8> = Vec::new();
    {
        let mut server = Server::with_diagnostics(&mut output, Diagnostics::Sink, db);
        server.register_tools();
        server.serve(&b"{not json}\n"[..]).expect("serve");
    }
    let replies = frames(&output);
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0]["error"]["code"], protocol::error_code::PARSE);
    assert_eq!(
        replies[0]["id"],
        Value::Null,
        "a parse error has no id to echo"
    );
    assert_eq!(replies[0]["jsonrpc"], "2.0");
}

#[test]
fn a_frame_above_the_cap_is_rejected_and_the_reader_resynchronises() {
    let (_dir, db) = test_db();
    let oversized = "x".repeat(transport::MAX_FRAME_BYTES + 64);
    let bad = format!("{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{oversized}\"}}\n");
    let good = format!(
        "{}\n",
        json!({ "jsonrpc": "2.0", "id": 2, "method": "ping" })
    );

    let mut output: Vec<u8> = Vec::new();
    {
        let mut server = Server::new(&mut output, db);
        server.register_tools();
        server
            .serve(format!("{bad}{good}").as_bytes())
            .expect("serve");
    }
    let replies = frames(&output);
    assert_eq!(
        replies.len(),
        2,
        "one error for the bad frame, one reply for the good one: {replies:?}"
    );
    assert!(
        replies[0]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("frame too large")
    );
    assert_eq!(replies[1]["id"], 2, "the reader did not resynchronise");
}

#[test]
fn params_that_are_not_an_object_report_invalid_params() {
    let (_dir, db) = test_db();
    let reply = single(request(36, "tools/call", json!("not an object")), db);
    assert_eq!(reply["error"]["code"], protocol::error_code::INVALID_PARAMS);
}

#[test]
fn call_params_with_no_arguments_name_no_tool_rather_than_failing_to_decode() {
    let (_dir, db) = test_db();
    let reply = single(request(37, "tools/call", Value::Null), db);
    assert_eq!(
        reply["error"]["code"],
        protocol::error_code::METHOD_NOT_FOUND
    );
}

#[test]
fn a_final_frame_without_a_trailing_newline_is_still_answered() {
    let (_dir, db) = test_db();
    let mut output: Vec<u8> = Vec::new();
    {
        let mut server = Server::new(&mut output, db);
        server.register_tools();
        let input = json!({ "jsonrpc": "2.0", "id": 38, "method": "ping" }).to_string();
        server.serve(input.as_bytes()).expect("serve");
    }
    let replies = frames(&output);
    assert_eq!(replies.len(), 1, "an unterminated final frame was dropped");
    assert_eq!(replies[0]["id"], 38);
}

#[test]
fn a_bad_jsonrpc_version_reports_invalid_request() {
    let (_dir, db) = test_db();
    let reply = single(json!({ "jsonrpc": "1.0", "id": 39, "method": "ping" }), db);
    assert_eq!(
        reply["error"]["code"],
        protocol::error_code::INVALID_REQUEST
    );
}

#[test]
fn a_request_without_a_method_reports_invalid_request() {
    let (_dir, db) = test_db();
    let reply = single(json!({ "jsonrpc": "2.0", "id": 40 }), db);
    assert_eq!(
        reply["error"]["code"],
        protocol::error_code::INVALID_REQUEST
    );
}

#[test]
fn every_reply_is_a_single_line_even_when_the_payload_is_not() {
    let (_dir, db) = test_db();
    let _collection = single(
        call_tool(
            41,
            "blazers_collections_create",
            json!({ "name": "multiline" }),
        ),
        db.clone(),
    );
    let mut output: Vec<u8> = Vec::new();
    {
        let mut server = Server::new(&mut output, db);
        server.register_tools();
        let input = format!(
            "{}\n{}\n",
            json!({ "jsonrpc": "2.0", "id": 42, "method": "tools/call", "params": {
                "name": "blazers_collections_list", "arguments": {}
            } }),
            json!({ "jsonrpc": "2.0", "id": 43, "method": "ping" }),
        );
        server.serve(input.as_bytes()).expect("serve");
    }
    let text = String::from_utf8(output.clone()).expect("utf8");
    assert_eq!(
        text.lines().count(),
        2,
        "each reply must be one line: {text}"
    );
    // The frame itself is compact even though the tool text is indented: the
    // payload's newlines appear only as escaped two-character sequences, so a
    // raw newline can never split a frame.
    assert!(
        text.contains("\\n  \\\"collections\\\""),
        "the indented payload should be escaped inside one frame: {text}"
    );
    assert_eq!(frames(&output).len(), 2);
}
