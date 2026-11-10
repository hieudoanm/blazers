use serde::Serialize;
use serde_json::{Map, Value};

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

    /// A property that is an array of strings.
    pub fn string_list(description: &str) -> Self {
        PropertySchema {
            kind: "array".to_string(),
            description: Some(description.to_string()),
            enum_values: None,
            items: Some(Box::new(PropertySchema::string("Item"))),
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
