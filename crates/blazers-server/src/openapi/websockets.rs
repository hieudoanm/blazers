pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/websockets": {
                "get": {"summary": "List WebSocket connections", "tags": ["WebSocket"], "responses": {"200": {"description": "Connection list"}}}
            },
            "/api/websockets/{id}": {
                "get": {"summary": "Get WS connection", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Connection details"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Close and delete WS connection", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/websockets/broadcast": {
                "post": {"summary": "Broadcast message to all WS clients", "tags": ["WebSocket"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"content": {"type": "string"}}, "required": ["content"]}}}}, "responses": {"200": {"description": "Broadcast result"}}}
            },
            "/api/websockets/{id}/send": {
                "post": {"summary": "Send message to a specific WS client", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"content": {"type": "string"}}, "required": ["content"]}}}}, "responses": {"200": {"description": "Message sent"}, "404": {"description": "Connection not found"}}}
            },
            "/api/websockets/{id}/messages": {
                "get": {"summary": "List messages for a WS connection", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Message list"}}}
            },
            "/api/websockets/messages": {
                "get": {"summary": "List all WS messages", "tags": ["WebSocket"], "responses": {"200": {"description": "All messages"}}}
            }
    }).as_object().unwrap().clone()
}
