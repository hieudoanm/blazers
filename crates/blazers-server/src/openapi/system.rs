pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/health": {
                "get": {
                    "summary": "Health check",
                    "tags": ["System"],
                    "security": [],
                    "responses": {
                        "200": {"description": "OK", "content": {"application/json": {"schema": {"type": "object", "properties": {"status": {"type": "string"}}}}}}
                    }
                }
            },
            "/api/openapi.json": {
                "get": {
                    "summary": "OpenAPI spec",
                    "tags": ["System"],
                    "security": [],
                    "responses": {"200": {"description": "OpenAPI JSON spec"}}
                }
            },
            "/api/docs": {
                "get": {
                    "summary": "Swagger UI",
                    "tags": ["System"],
                    "security": [],
                    "responses": {"200": {"description": "Swagger UI HTML"}}
                }
            },
            "/api/backup": {
                "get": {
                    "summary": "Download SQLite database backup",
                    "tags": ["System"],
                    "responses": {"200": {"description": "Database file download", "content": {"application/octet-stream": {}}}}
                }
            },
            "/api/backup/run": {
                "post": {
                    "summary": "Run a backup immediately (admin)",
                    "tags": ["System"],
                    "responses": {"200": {"description": "Backup written", "content": {"application/json": {"schema": {"type": "object", "properties": {"path": {"type": "string"}}}}}}, "403": {"description": "Forbidden"}}
                }
            },
            "/api/settings": {
                "get": {
                    "summary": "List settings (smtp_pass masked)",
                    "tags": ["System"],
                    "responses": {"200": {"description": "Settings map"}}
                },
                "put": {
                    "summary": "Set settings: smtp_host, smtp_port, smtp_user, smtp_from, smtp_pass (encrypted), app_url, backup_schedule, backup_dir, backup_keep",
                    "tags": ["System"],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "additionalProperties": true}}}},
                    "responses": {"200": {"description": "Updated settings"}}
                }
            },
            "/api/settings/{key}": {
                "delete": {
                    "summary": "Delete a setting",
                    "tags": ["System"],
                    "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}
                }
            }
    }).as_object().unwrap().clone()
}
