pub fn paths() -> serde_json::Map<String, serde_json::Value> {
    serde_json::json!({
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
            "/api/cache/{key}/incr": {
                "post": {"summary": "Atomically increment an integer value", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"content": {"application/json": {"schema": {"type": "object", "properties": {"by": {"type": "integer"}}}}}}, "responses": {"200": {"description": "New value"}, "400": {"description": "WRONGTYPE"}}}
            },
            "/api/cache/{key}/append": {
                "post": {"summary": "Atomically append to a string value", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"value": {"type": "string"}}, "required": ["value"]}}}}, "responses": {"200": {"description": "Full string"}, "400": {"description": "WRONGTYPE"}}}
            },
            "/api/cache/{key}/expire": {
                "post": {"summary": "Set TTL in seconds (0 clears)", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"ttl": {"type": "integer"}}, "required": ["ttl"]}}}}, "responses": {"200": {"description": "TTL set"}, "404": {"description": "Not found"}}}
            },
            "/api/cache/{key}/ttl": {
                "get": {"summary": "Get TTL (-1 = no expiry)", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "TTL"}, "404": {"description": "Not found"}}}
            },
            "/api/cache/{key}/list": {
                "get": {"summary": "LRANGE — list slice (start/stop, inclusive, negatives from end)", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "start", "in": "query", "schema": {"type": "integer"}}, {"name": "stop", "in": "query", "schema": {"type": "integer"}}], "responses": {"200": {"description": "Items"}, "400": {"description": "WRONGTYPE"}}},
                "post": {"summary": "RPUSH/LPUSH values onto a list", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"required": true, "content": {"application/json": {"schema": {"type": "object", "properties": {"values": {"type": "array"}, "left": {"type": "boolean"}}, "required": ["values"]}}}}, "responses": {"200": {"description": "New length"}, "400": {"description": "WRONGTYPE"}}}
            },
            "/api/cache/{key}/list/pop": {
                "post": {"summary": "LPOP/RPOP items from a list", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"content": {"application/json": {"schema": {"type": "object", "properties": {"left": {"type": "boolean"}, "count": {"type": "integer"}}}}}}, "responses": {"200": {"description": "Popped value(s)"}, "400": {"description": "WRONGTYPE"}}}
            },
            "/api/cache/{key}/hash": {
                "get": {"summary": "HGETALL — all hash fields", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Fields"}, "400": {"description": "WRONGTYPE"}}},
                "post": {"summary": "HSET — set hash field(s)", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}], "requestBody": {"content": {"application/json": {"schema": {"type": "object", "properties": {"field": {"type": "string"}, "value": {}, "fields": {"type": "object"}}}}}}, "responses": {"200": {"description": "Fields set"}, "400": {"description": "WRONGTYPE or empty fields"}}}
            },
            "/api/cache/{key}/hash/{field}": {
                "get": {"summary": "HGET — one hash field", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "field", "in": "path", "required": true, "schema": {"type": "string"}}], "responses": {"200": {"description": "Field value"}, "400": {"description": "WRONGTYPE"}}},
                "delete": {"summary": "HDEL — delete hash field(s)", "tags": ["Cache"], "parameters": [{"name": "key", "in": "path", "required": true, "schema": {"type": "string"}}, {"name": "field", "in": "path", "required": true, "schema": {"type": "string", "description": "Comma-separated fields"}}], "responses": {"200": {"description": "Removed count"}, "400": {"description": "WRONGTYPE"}}}
            }
    }).as_object().unwrap().clone()
}
