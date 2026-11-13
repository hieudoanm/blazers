pub fn build_openapi_json() -> serde_json::Value {
    let mut paths = serde_json::Map::new();
    for group in [
        system::paths(),
        auth::paths(),
        collections::paths(),
        buckets::paths(),
        webhooks::paths(),
        secrets::paths(),
        websockets::paths(),
        cache::paths(),
        notifications::paths(),
        logs::paths(),
        pubsub::paths(),
        cronjobs::paths(),
        functions::paths(),
    ] {
        paths.extend(group);
    }
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
        "paths": serde_json::Value::Object(paths),
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

mod auth;
mod buckets;
mod cache;
mod collections;
mod cronjobs;
mod functions;
mod logs;
mod notifications;
mod pubsub;
mod secrets;
mod system;
mod webhooks;
mod websockets;

#[cfg(test)]
mod tests;
