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
            }
    }).as_object().unwrap().clone()
}
