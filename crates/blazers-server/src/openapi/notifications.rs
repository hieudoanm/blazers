pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/notifications": {
                "get": {"summary": "List notifications", "tags": ["Notifications"], "responses": {"200": {"description": "Notification list"}}},
                "post": {"summary": "Create notification", "tags": ["Notifications"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"title": {"type": "string"}, "body": {"type": "string"}, "type": {"type": "string", "enum": ["info", "success", "warning", "error"]}}, "required": ["title"]}}}}, "responses": {"200": {"description": "Created notification"}, "400": {"description": "Invalid title or type"}}},
                "delete": {"summary": "Clear all notifications", "tags": ["Notifications"], "responses": {"200": {"description": "Cleared"}}}
            },
            "/api/notifications/{id}": {
                "get": {"summary": "Get notification", "tags": ["Notifications"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Notification details"}, "404": {"description": "Not found"}}},
                "patch": {"summary": "Mark notification as read", "tags": ["Notifications"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Updated notification"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete notification", "tags": ["Notifications"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/notifications/stream": {
                "get": {"summary": "SSE stream for notifications", "tags": ["Notifications"], "responses": {"200": {"description": "SSE event stream"}}}
            }
    }).as_object().unwrap().clone()
}
