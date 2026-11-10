use super::*;

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
