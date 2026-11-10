pub fn build_openapi_json() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Blazers Server API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "A blazers REST API server with collections, buckets, webhooks, secrets, cache, notifications, logs, pub/sub, cron jobs, and WebSocket support."
        },
        "servers": [{"url": "/", "description": "Local server"}],
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "paths": {
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
            },
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
            },
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
                "get": {"summary": "Download file", "tags": ["Files"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "File binary content"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete file", "tags": ["Files"], "parameters": [{"name": "name", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
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
            },
            "/api/secrets": {
                "get": {"summary": "List secrets (values hidden)", "tags": ["Secrets"], "responses": {"200": {"description": "Secret list without values"}}},
                "post": {"summary": "Create secret", "tags": ["Secrets"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"name": {"type": "string"}, "value": {"type": "string"}, "scope": {"type": "string"}}, "required": ["name"]}}}}, "responses": {"201": {"description": "Created secret"}, "400": {"description": "Missing name"}}}
            },
            "/api/secrets/{id}": {
                "get": {"summary": "Get secret (decrypted)", "tags": ["Secrets"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Secret with value"}, "404": {"description": "Not found"}}},
                "patch": {"summary": "Update secret", "tags": ["Secrets"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"name": {"type": "string"}, "value": {"type": "string"}, "scope": {"type": "string"}}}}}}, "responses": {"200": {"description": "Updated secret"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete secret", "tags": ["Secrets"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/websockets": {
                "get": {"summary": "List WebSocket connections", "tags": ["WebSocket"], "responses": {"200": {"description": "Connection list"}}}
            },
            "/api/websockets/{id}": {
                "get": {"summary": "Get WS connection", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Connection details"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Close and delete WS connection", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
            "/api/websockets/broadcast": {
                "post": {"summary": "Broadcast message to all WS clients", "tags": ["WebSocket"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"content": {"type": "string"}}, "required": ["content"]}}}}, "responses": {"200": {"description": "Broadcast result"}}}
            },
            "/api/websockets/{id}/send": {
                "post": {"summary": "Send message to a specific WS client", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"content": {"type": "string"}}, "required": ["content"]}}}}, "responses": {"200": {"description": "Message sent"}, "404": {"description": "Connection not found"}}}
            },
            "/api/websockets/{id}/messages": {
                "get": {"summary": "List messages for a WS connection", "tags": ["WebSocket"], "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Message list"}}}
            },
            "/api/websockets/messages": {
                "get": {"summary": "List all WS messages", "tags": ["WebSocket"], "responses": {"200": {"description": "All messages"}}}
            },
            "/api/cache": {
                "get": {"summary": "List cache entries", "tags": ["Cache"], "responses": {"200": {"description": "Cache list"}}},
                "post": {"summary": "Set cache entry", "tags": ["Cache"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"key": {"type": "string"}, "value": {"type": "string"}, "ttl": {"type": "integer", "description": "TTL in seconds"}}, "required": ["key", "value"]}}}}, "responses": {"200": {"description": "Cache entry created"}, "400": {"description": "Missing key/value"}}},
                "delete": {"summary": "Flush all cache", "tags": ["Cache"], "responses": {"200": {"description": "Cache flushed"}}}
            },
            "/api/cache/stats": {
                "get": {"summary": "Get cache statistics", "tags": ["Cache"], "responses": {"200": {"description": "Cache stats"}}}
            },
            "/api/cache/{key}": {
                "get": {"summary": "Get cache entry", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Cache entry"}, "404": {"description": "Not found"}}},
                "delete": {"summary": "Delete cache entry", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"204": {"description": "Deleted"}, "404": {"description": "Not found"}}}
            },
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
            },
            "/api/logs": {
                "get": {"summary": "List logs", "tags": ["Logs"], "responses": {"200": {"description": "Log list"}}},
                "post": {"summary": "Create log entry", "tags": ["Logs"], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"level": {"type": "string", "enum": ["debug", "info", "warn", "error"]}, "message": {"type": "string"}, "meta": {"type": "string"}}, "required": ["message"]}}}}, "responses": {"200": {"description": "Created log"}, "400": {"description": "Invalid level or empty message"}}},
                "delete": {"summary": "Clear all logs", "tags": ["Logs"], "responses": {"200": {"description": "Cleared"}}}
            },
            "/api/logs/stream": {
                "get": {"summary": "SSE stream for logs", "tags": ["Logs"], "responses": {"200": {"description": "SSE event stream"}}}
            },
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
            },
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
            },
            "/api/backup": {
                "get": {
                    "summary": "Download SQLite database backup",
                    "tags": ["System"],
                    "responses": {"200": {"description": "Database file download", "content": {"application/octet-stream": {}}}}
                }
            }
        },
        "schemas": {
            "Collection": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "schema": {"type": "string"},
                    "created_at": {"type": "string"},
                    "updated_at": {"type": "string"}
                }
            },
            "Record": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "data": {"type": "object"},
                    "created_at": {"type": "string"},
                    "updated_at": {"type": "string"}
                }
            },
            "RecordsPage": {
                "type": "object",
                "properties": {
                    "records": {"type": "array", "items": {"$ref": "#/components/schemas/Record"}},
                    "total": {"type": "integer"},
                    "page": {"type": "integer"},
                    "per_page": {"type": "integer"},
                    "total_pages": {"type": "integer"}
                }
            },
            "CreateWebhookRequest": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "url": {"type": "string"},
                    "events": {"type": "array", "items": {"type": "string"}},
                    "secret": {"type": "string"}
                },
                "required": ["name", "url", "events"]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_json_object() {
        let spec = build_openapi_json();
        assert!(spec.is_object());
    }

    #[test]
    fn has_openapi_version() {
        let spec = build_openapi_json();
        assert_eq!(spec["openapi"], "3.0.0");
    }

    #[test]
    fn has_paths_field() {
        let spec = build_openapi_json();
        assert!(spec.get("paths").is_some());
    }

    #[test]
    fn paths_contains_health() {
        let spec = build_openapi_json();
        assert!(spec["paths"].get("/api/health").is_some());
    }

    #[test]
    fn has_info_with_title_and_version() {
        let spec = build_openapi_json();
        let info = spec.get("info").unwrap();
        assert!(info.get("title").is_some());
        assert!(info.get("version").is_some());
    }

    #[test]
    fn has_components_field() {
        let spec = build_openapi_json();
        assert!(spec.get("components").is_some());
    }
}
