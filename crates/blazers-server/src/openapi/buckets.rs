pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/buckets": {
                "get": {"summary": "List buckets", "tags": ["Buckets"], "responses": {"200": {"description": "Bucket list"}}},
                "post": {"summary": "Create bucket", "tags": ["Buckets"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"name": {"type": "string"}, "is_public": {"type": "boolean"}}, "required": ["name"]}}}}, "responses": {"200": {"description": "Created bucket"}, "409": {"description": "Already exists"}}}
            },
            "/api/buckets/{name}": {
                "get": {"summary": "Get bucket", "tags": ["Buckets"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Bucket details"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete bucket", "tags": ["Buckets"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/buckets/{name}/files": {
                "get": {"summary": "List files", "tags": ["Files"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "page", "in": "query", "schema": {"type": "integer"}}, {"name": "per_page", "in": "query", "schema": {"type": "integer"}}], "responses": {"200": {"description": "Paginated files"}}},
                "post": {"summary": "Upload file", "tags": ["Files"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"multipart/form-data": {"schema": {"type": "object", "properties": {"file": {"type": "string", "format": "binary"}}}}}}, "responses": {"200": {"description": "Uploaded file metadata"}}}
            },
            "/api/buckets/{name}/files/{id}": {
                "get": {"summary": "Download file (public buckets and ?token= file tokens allowed without auth)", "tags": ["Files"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "token", "in": "query", "schema": {"type": "string"}, "description": "File access token"}], "responses": {"200": {"description": "File binary content"}, "401": {"description": "Auth required"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete file", "tags": ["Files"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/buckets/{name}/files/{id}/token": {
                "post": {"summary": "Issue a 1h download token for a protected file; optional {collection, record} body binds issuance to that record's view_rule", "tags": ["Files"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"content": {"application/json": {"schema": {"type": "object", "properties": {"collection": {"type": "string"}, "record": {"type": "string"}}}}}}, "responses": {"200": {"description": "{token, expires_in}"}, "401": {"description": "Unauthorized"}, "403": {"description": "Record view_rule denied"}}}
            }
    }).as_object().unwrap().clone()
}
