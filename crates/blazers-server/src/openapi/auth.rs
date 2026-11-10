pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
            "/api/auth/register": {
                "post": {
                    "summary": "Register a new user",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "email": {"type": "string", "format": "email"},
                        "password": {"type": "string", "minLength": 6}
                    }, "required": ["email", "password"]}}}},
                    "responses": {
                        "200": {"description": "User registered"},
                        "400": {"description": "Bad request"}
                    }
                }
            },
            "/api/auth/login": {
                "post": {
                    "summary": "Login and get JWT token",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {
                        "email": {"type": "string"},
                        "password": {"type": "string"}
                    }, "required": ["email", "password"]}}}},
                    "responses": {
                        "200": {"description": "Login success, returns token"},
                        "401": {"description": "Invalid credentials"}
                    }
                }
            }
    }).as_object().unwrap().clone()
}
