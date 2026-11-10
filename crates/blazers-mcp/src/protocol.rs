//! The JSON-RPC and MCP wire types this server speaks.
//!
//! The transport is one JSON document per line, so a reply is always a single
//! line even when the payload it carries is pretty-printed.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The JSON-RPC dialect every frame must declare.
pub const JSONRPC_VERSION: &str = "2.0";

/// The MCP revision this server implements.
pub const PROTOCOL_VERSION: &str = "2025-11-25";

/// JSON-RPC 2.0 error codes used by this server.
pub mod error_code {
    pub const PARSE: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL: i64 = -32603;
}

/// An incoming request frame. A missing or JSON-null id marks a notification.
#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub jsonrpc: Option<String>,
    #[serde(default)]
    pub id: Option<Value>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub params: Option<Value>,
}

impl Request {
    /// Reports whether this frame expects no reply.
    ///
    /// A *string* `"null"` is a real id, not a notification: a client may use it
    /// as a request key, and dropping the reply would hang that request.
    pub fn is_notification(&self) -> bool {
        matches!(self.id, None | Some(Value::Null))
    }
}

/// One property in a tool's JSON schema.
#[derive(Debug, Clone, Serialize)]
pub struct PropertySchema {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertySchema>>,
}

impl PropertySchema {
    /// A property that is always a string.
    pub fn string(description: &str) -> Self {
        PropertySchema {
            kind: "string".to_string(),
            description: Some(description.to_string()),
            enum_values: None,
            items: None,
        }
    }

    /// A property that is always an object, such as a record body.
    pub fn object(description: &str) -> Self {
        PropertySchema {
            kind: "object".to_string(),
            description: Some(description.to_string()),
            enum_values: None,
            items: None,
        }
    }

    /// A property that is always an integer.
    pub fn integer(description: &str) -> Self {
        PropertySchema {
            kind: "integer".to_string(),
            description: Some(description.to_string()),
            enum_values: None,
            items: None,
        }
    }

    /// A property restricted to a fixed set of strings.
    pub fn enumeration(description: &str, values: &[&str]) -> Self {
        PropertySchema {
            kind: "string".to_string(),
            description: Some(description.to_string()),
            enum_values: Some(values.iter().map(|v| v.to_string()).collect()),
            items: None,
        }
    }
}

/// A tool's input schema.
#[derive(Debug, Clone, Serialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub kind: String,
    pub properties: Map<String, Value>,
    pub required: Vec<String>,
}

impl Schema {
    /// A schema taking no arguments.
    pub fn empty() -> Self {
        Schema {
            kind: "object".to_string(),
            properties: Map::new(),
            required: Vec::new(),
        }
    }

    /// A schema built from properties, of which `required` are mandatory.
    pub fn new(properties: Vec<(&str, PropertySchema)>, required: &[&str]) -> Self {
        let mut map = Map::new();
        for (name, schema) in properties {
            map.insert(
                name.to_string(),
                serde_json::to_value(schema).expect("a schema is always serialisable"),
            );
        }
        Schema {
            kind: "object".to_string(),
            properties: map,
            required: required.iter().map(|r| r.to_string()).collect(),
        }
    }
}

/// One tool exposed over MCP.
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Schema,
}

/// A text block inside a tool result.
#[derive(Debug, Clone, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

/// What a tool handler returns. A failure sets `isError` rather than panicking,
/// so a model can correct the call instead of the session dying.
#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    pub content: Vec<ContentItem>,
    /// Omitted on success, so a model reads only real failures.
    #[serde(rename = "isError", skip_serializing_if = "is_false")]
    pub is_error: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl ToolResult {
    /// Text content, for a successful tool.
    pub fn text(body: impl Into<String>) -> Self {
        ToolResult {
            content: vec![ContentItem {
                kind: "text".to_string(),
                text: body.into(),
            }],
            is_error: false,
        }
    }

    /// Errored content, for a tool the model should not treat as a success.
    pub fn error(message: impl Into<String>) -> Self {
        ToolResult {
            content: vec![ContentItem {
                kind: "text".to_string(),
                text: message.into(),
            }],
            is_error: true,
        }
    }
}

/// An error object in a JSON-RPC response.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorObject {
    pub code: i64,
    pub message: String,
}

/// A JSON-RPC response. Exactly one of result and error is populated.
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObject>,
}

/// The reply to a tools/list request.
#[derive(Debug, Serialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

/// The arguments of a tools/call request.
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

impl ToolCallParams {
    /// Returns the arguments as text for a handler. Absent arguments become an
    /// empty object so a handler sees a well-formed value.
    pub fn arguments_json(&self) -> String {
        match &self.arguments {
            None | Some(Value::Null) => "{}".to_string(),
            Some(value) => value.to_string(),
        }
    }
}

/// Text content, for a successful tool.
pub fn text_result(body: impl Into<String>) -> ToolResult {
    ToolResult::text(body)
}

/// Errored content, for a tool the model should not treat as a success.
pub fn error_result(message: impl Into<String>) -> ToolResult {
    ToolResult::error(message)
}

/// A successful response carrying `result`.
pub fn success_response(id: Value, result: Value) -> Response {
    Response {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

/// An error response.
pub fn error_response(id: Value, code: i64, message: impl Into<String>) -> Response {
    Response {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(ErrorObject {
            code,
            message: message.into(),
        }),
    }
}

/// Decodes tools/call params, treating absent or null params as an empty object
/// so a call with no params names no tool rather than failing to decode.
pub fn object_or_empty(params: Option<&Value>) -> Result<ToolCallParams, String> {
    let value = match params {
        None | Some(Value::Null) => serde_json::json!({}),
        Some(value) => value.clone(),
    };
    if !value.is_object() {
        return Err("params must be an object".to_string());
    }
    serde_json::from_value(value).map_err(|err| err.to_string())
}

/// Builds the initialize reply.
///
/// This server speaks exactly one revision, so it always answers with
/// [PROTOCOL_VERSION] whether or not the client asked for it. A client that
/// requested something else is expected to disconnect after seeing the
/// mismatch, which is what the specification prescribes.
pub fn initialize_result(_params: Option<&Value>, name: &str, version: &str) -> Value {
    serde_json::json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": name, "version": version },
    })
}
