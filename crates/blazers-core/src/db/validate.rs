use serde_json::Value;

use crate::models::*;

pub(super) fn validate_field(value: &Value, type_str: &str, field_name: &str) -> Option<String> {
    match type_str {
        "string" => {
            if !value.is_string() {
                Some(format!("field '{field_name}' must be a string"))
            } else {
                None
            }
        }
        "number" => {
            if !value.is_number() {
                Some(format!("field '{field_name}' must be a number"))
            } else {
                None
            }
        }
        "integer" => match value.as_f64() {
            Some(f) if f.fract() == 0.0 => None,
            _ => Some(format!("field '{field_name}' must be an integer")),
        },
        "boolean" => {
            if !value.is_boolean() {
                Some(format!("field '{field_name}' must be a boolean"))
            } else {
                None
            }
        }
        "array" => {
            if !value.is_array() {
                Some(format!("field '{field_name}' must be an array"))
            } else {
                None
            }
        }
        "object" => {
            if !value.is_object() {
                Some(format!("field '{field_name}' must be an object"))
            } else {
                None
            }
        }
        "email" => match value.as_str() {
            Some(s) if s.contains('@') => None,
            _ => Some(format!("field '{field_name}' must be a valid email")),
        },
        "url" => match value.as_str() {
            Some(s) if s.starts_with("http://") || s.starts_with("https://") => None,
            _ => Some(format!("field '{field_name}' must be a valid URL")),
        },
        _ => None,
    }
}

const SCHEMA_TYPES: &[&str] = &[
    "string", "number", "integer", "boolean", "array", "object", "email", "url",
];

/// Validates a collection `schema` string: must be a JSON object mapping
/// field names (optionally suffixed with `?` for optional) to a known type.
pub fn validate_schema(schema: &str) -> Result<()> {
    if schema.is_empty() || schema == "{}" {
        return Ok(());
    }
    let map: serde_json::Map<String, Value> = serde_json::from_str(schema)
        .map_err(|e| AppError::BadRequest(format!("invalid schema JSON: {e}")))?;
    for (field, type_val) in &map {
        let base = field.strip_suffix('?').unwrap_or(field);
        if base.is_empty() {
            return Err(AppError::BadRequest("schema field name is empty".into()));
        }
        let ty = type_val.as_str().ok_or_else(|| {
            AppError::BadRequest(format!("schema field '{field}' must map to a type string"))
        })?;
        if !SCHEMA_TYPES.contains(&ty) {
            return Err(AppError::BadRequest(format!(
                "schema field '{field}' has unknown type '{ty}' (expected one of: {})",
                SCHEMA_TYPES.join(", ")
            )));
        }
    }
    Ok(())
}

pub fn validate_data(data: &Value, schema: &str) -> Result<()> {
    if schema.is_empty() || schema == "{}" {
        return Ok(());
    }
    let schema_map: serde_json::Map<String, Value> = serde_json::from_str(schema)
        .map_err(|e| AppError::Internal(format!("parse schema: {e}")))?;
    let mut errors = Vec::new();
    for (field, type_val) in &schema_map {
        let type_str = type_val.as_str().unwrap_or("string");
        let is_optional = field.ends_with('?');
        let field_name = if is_optional {
            &field[..field.len() - 1]
        } else {
            field.as_str()
        };
        if is_optional && data.get(field_name).is_none_or(|v| v.is_null()) {
            continue;
        }
        match data.get(field_name) {
            None => {
                if !is_optional {
                    errors.push(format!("field '{field_name}' is required"));
                }
            }
            Some(v) if v.is_null() && is_optional => {}
            Some(v) => {
                if let Some(err) = validate_field(v, type_str, field_name) {
                    errors.push(err);
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::BadRequest(errors.join("; ")))
    }
}
