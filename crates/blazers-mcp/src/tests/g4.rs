use super::*;

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
