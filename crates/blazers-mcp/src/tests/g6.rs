use super::*;

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
