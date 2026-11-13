//! OpenAPI paths for edge functions (`/api/functions`).

pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
        "/api/functions": {
            "get": {
                "summary": "List edge functions (admin)",
                "tags": ["Functions"],
                "responses": {"200": {"description": "Array of stored functions"}, "403": {"description": "Admin role required"}}
            },
            "post": {
                "summary": "Create or replace an edge function (admin)",
                "description": "Stores JS source executed by the embedded QuickJS runtime. The function's `handler`/`main` receives `{ \"input\": <request body input>, \"auth\": {user_id, email} | null }` and returns a JSON value. No I/O, timers, or host bindings.",
                "tags": ["Functions"],
                "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                    "name": {"type": "string"},
                    "code": {"type": "string", "description": "JavaScript source defining handler(input) or main(input)"}
                }, "required": ["name", "code"]}}}},
                "responses": {"200": {"description": "Stored function"}, "400": {"description": "name/code required"}, "403": {"description": "Admin role required"}}
            }
        },
        "/api/functions/{name}": {
            "post": {
                "summary": "Invoke an edge function",
                "description": "Body: `{ \"input\": <any JSON> }`. Returns `{ \"result\": <handler return value> }`. Send a Bearer token to expose the caller's identity as `auth` in the handler context.",
                "tags": ["Functions"],
                "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}],
                "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"input": {}}}}}},
                "responses": {
                    "200": {"description": "Function result"},
                    "400": {"description": "JS error or missing handler"},
                    "404": {"description": "Function not found"}
                }
            },
            "delete": {
                "summary": "Delete an edge function (admin)",
                "tags": ["Functions"],
                "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}],
                "responses": {"204": {"description": "Deleted"}, "403": {"description": "Admin role required"}, "404": {"description": "Function not found"}}
            }
        }
    })
    .as_object()
    .unwrap()
    .clone()
}
