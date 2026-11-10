//! Protocol behaviour of the Blazers MCP server.
//!
//! Each test drives the real [super::Server] against a real migrated SQLite
//! database in a temporary directory, so tool handlers are exercised against
//! real storage rather than a stub. `BLAZERS_DATA` is pointed at that directory
//! so the handlers and `mcp::run` open the same file.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde_json::{Value, json};

use super::{Diagnostics, SERVER_VERSION, Server, protocol, transport};
use crate::handlers::DbHandle;
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

mod g1;
mod g2;
mod g3;
mod g4;
mod g5;
mod g6;
