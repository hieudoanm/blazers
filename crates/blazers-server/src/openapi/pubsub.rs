pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/pubsub/topics": {
                "get": {"summary": "List pub/sub topics", "tags": ["Pub/Sub"], "responses": {"200": {"description": "Topic list"}}},
                "post": {"summary": "Create pub/sub topic", "tags": ["Pub/Sub"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"name": {"type": "string"}}, "required": ["name"]}}}}, "responses": {"201": {"description": "Created topic"}, "409": {"description": "Already exists"}}}
            },
            "/api/pubsub/topics/{name}": {
                "get": {"summary": "Get pub/sub topic", "tags": ["Pub/Sub"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Topic details"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete pub/sub topic", "tags": ["Pub/Sub"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/pubsub/topics/{name}/messages": {
                "get": {"summary": "List pub/sub messages", "tags": ["Pub/Sub"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Message list"}}},
                "post": {"summary": "Publish message to topic", "tags": ["Pub/Sub"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"body": {"type": "string"}}, "required": ["body"]}}}}, "responses": {"201": {"description": "Published message"}, "400": {"description": "Missing body"}}}
            },
            "/api/pubsub/{name}/stream": {
                "get": {"summary": "SSE stream for pub/sub topic", "tags": ["Pub/Sub"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "SSE event stream"}}}
            }
    }).as_object().unwrap().clone()
}
