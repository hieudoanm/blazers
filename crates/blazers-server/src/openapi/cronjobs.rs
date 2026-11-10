pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/cronjobs": {
                "get": {"summary": "List cron jobs", "tags": ["Cron Jobs"], "responses": {"200": {"description": "Cron job list"}}},
                "post": {"summary": "Create cron job", "tags": ["Cron Jobs"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                    "name": {"type": "string"},
                    "schedule": {"type": "string", "description": "Cron expression"},
                    "command": {"type": "string", "description": "URL to call"},
                    "method": {"type": "string", "enum": ["GET", "POST", "PUT", "PATCH", "DELETE"]},
                    "headers": {"type": "string"},
                    "body": {"type": "string"}
                }, "required": ["name", "schedule", "command"]}}}}, "responses": {"201": {"description": "Created cron job"}}}
            },
            "/api/cronjobs/{id}": {
                "get": {"summary": "Get cron job", "tags": ["Cron Jobs"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Cron job details"}, "404": {"description": "Not found"}}},
                "patch": {"summary": "Update cron job", "tags": ["Cron Jobs"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                    "name": {"type": "string"},
                    "schedule": {"type": "string"},
                    "command": {"type": "string"},
                    "method": {"type": "string"},
                    "headers": {"type": "string"},
                    "body": {"type": "string"},
                    "is_active": {"type": "boolean"}
                }}}}}, "responses": {"200": {"description": "Updated cron job"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete cron job", "tags": ["Cron Jobs"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/cronjobs/{id}/run": {
                "post": {"summary": "Trigger cron job execution", "tags": ["Cron Jobs"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Triggered"}}}
            },
            "/api/cronjobs/{id}/logs": {
                "get": {"summary": "List cron job execution logs", "tags": ["Cron Jobs"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Execution logs"}}}
            }
    }).as_object().unwrap().clone()
}
