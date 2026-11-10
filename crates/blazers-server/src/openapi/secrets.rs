pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/secrets": {
                "get": {"summary": "List secrets (values hidden)", "tags": ["Secrets"], "responses": {"200": {"description": "Secret list without values"}}},
                "post": {"summary": "Create secret", "tags": ["Secrets"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"name": {"type": "string"}, "value": {"type": "string"}, "scope": {"type": "string"}}, "required": ["name"]}}}}, "responses": {"201": {"description": "Created secret"}, "400": {"description": "Missing name"}}}
            },
            "/api/secrets/{id}": {
                "get": {"summary": "Get secret (decrypted)", "tags": ["Secrets"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Secret with value"}, "404": {"description": "Not found"}}},
                "patch": {"summary": "Update secret", "tags": ["Secrets"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"name": {"type": "string"}, "value": {"type": "string"}, "scope": {"type": "string"}}}}}}, "responses": {"200": {"description": "Updated secret"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete secret", "tags": ["Secrets"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            }
    }).as_object().unwrap().clone()
}
