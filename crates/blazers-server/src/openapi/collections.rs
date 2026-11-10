pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/collections": {
                "get": {
                    "summary": "List all collections",
                    "tags": ["Collections"],
                    "responses": {"200": {"description": "List of collections", "content": {"application/json": {"schema": {"type": "array", "items": {"$ref": "#/components/schemas/Collection"}}}}}}
                },
                "post": {
                    "summary": "Create a collection",
                    "tags": ["Collections"],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "name": {"type": "string"},
                        "schema": {"type": "string", "description": "JSON schema string"}
                    }, "required": ["name"]}}}},
                    "responses": {"200": {"description": "Created collection"}, "409": {"description": "Already exists"}}
                }
            },
            "/api/collections/{name}": {
                "get": {
                    "summary": "Get collection by name",
                    "tags": ["Collections"],
                    "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": {"200": {"description": "Collection details"}, "404": {"description": "Not found"}}
                },
                "patch": {
                    "summary": "Update collection schema",
                    "tags": ["Collections"],
                    "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "schema": {"type": "string", "description": "New JSON schema"}
                    }}}}},
                    "responses": {"200": {"description": "Updated collection"}, "404": {"description": "Not found"}}
                },
                "delete": {
                    "summary": "Delete a collection",
                    "tags": ["Collections"],
                    "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}
                }
            },
            "/api/collections/{name}/records": {
                "get": {
                    "summary": "List records with pagination and search",
                    "tags": ["Records"],
                    "parameters": [
                        {"name": "name", "in": "path", "required": true, "schema": {"type": "string"}},
                        {"name": "page", "in": "query", "schema": {"type": "integer", "default": 1}},
                        {"name": "per_page", "in": "query", "schema": {"type": "integer", "default": 20}},
                        {"name": "search", "in": "query", "schema": {"type": "string"}, "description": "Full-text search on data field"}
                    ],
                    "responses": {"200": {"description": "Paginated records", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/RecordsPage"}}}}}
                },
                "post": {
                    "summary": "Create a record",
                    "tags": ["Records"],
                    "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "id": {"type": "string", "description": "Optional custom ID"},
                        "data": {"type": "object", "description": "Record data"}
                    }}}}},
                    "responses": {"200": {"description": "Created record"}}
                }
            },
            "/api/collections/{name}/records/{id}": {
                "get": {
                    "summary": "Get a record by ID",
                    "tags": ["Records"],
                    "parameters": [
                        {"name": "name", "in": "path", "required": true, "schema": {"type": "string"}},
                        {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}
                    ],
                    "responses": {"200": {"description": "Record details"}, "404": {"description": "Not found"}}
                },
                "patch": {
                    "summary": "Update a record",
                    "tags": ["Records"],
                    "parameters": [
                        {"name": "name", "in": "path", "required": true, "schema": {"type": "string"}},
                        {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}
                    ],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "data": {"type": "object"}
                    }, "required": ["data"]}}}},
                    "responses": {"200": {"description": "Updated record"}, "404": {"description": "Not found"}}
                },
                "delete": {
                    "summary": "Delete a record",
                    "tags": ["Records"],
                    "parameters": [
                        {"name": "name", "in": "path", "required": true, "schema": {"type": "string"}},
                        {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}
                    ],
                    "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}
                }
            }
    }).as_object().unwrap().clone()
}
