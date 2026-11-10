pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/logs": {
                "get": {"summary": "List logs", "tags": ["Logs"], "responses": {"200": {"description": "Log list"}}},
                "post": {"summary": "Create log entry", "tags": ["Logs"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"level": {"type": "string", "enum": ["debug", "info", "warn", "error"]}, "message": {"type": "string"}, "meta": {"type": "string"}}, "required": ["message"]}}}}, "responses": {"200": {"description": "Created log"}, "400": {"description": "Invalid level or empty message"}}},
                "delete": {"summary": "Clear all logs", "tags": ["Logs"], "responses": {"200": {"description": "Cleared"}}}
            },
            "/api/logs/stream": {
                "get": {"summary": "SSE stream for logs", "tags": ["Logs"], "responses": {"200": {"description": "SSE event stream"}}}
            }
    }).as_object().unwrap().clone()
}
