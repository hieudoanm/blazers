pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/webhooks": {
                "get": {"summary": "List webhooks", "tags": ["Webhooks"], "responses": {"200": {"description": "Webhook list"}}},
                "post": {"summary": "Create webhook", "tags": ["Webhooks"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/CreateWebhookRequest"}}}}, "responses": {"200": {"description": "Created webhook"}, "400": {"description": "Validation error"}}}
            },
            "/api/webhooks/{id}": {
                "get": {"summary": "Get webhook", "tags": ["Webhooks"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Webhook details"}, "404": {"description": "Not found"}}},
                "patch": {"summary": "Update webhook", "tags": ["Webhooks"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"name": {"type": "string"}, "url": {"type": "string"}, "events": {"type": "array", "items": {"type": "string"}}, "is_active": {"type": "boolean"}}}}}}, "responses": {"200": {"description": "Updated webhook"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete webhook", "tags": ["Webhooks"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/webhooks/{id}/logs": {
                "get": {"summary": "List webhook execution logs", "tags": ["Webhooks"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Webhook logs"}}}
            }
    }).as_object().unwrap().clone()
}
