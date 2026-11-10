use super::*;

pub(crate) enum CacheValue {
    Str(String),
    List(Vec<serde_json::Value>),
    Hash(serde_json::Map<String, serde_json::Value>),
}

pub(crate) fn decode_value(raw: &str) -> CacheValue {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw)
        && let Some(obj) = v.as_object()
        && obj.len() == 2
    {
        match obj.get("__type").and_then(|t| t.as_str()) {
            Some("list") => {
                if let Some(items) = obj.get("items").and_then(|i| i.as_array()) {
                    return CacheValue::List(items.clone());
                }
            }
            Some("hash") => {
                if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
                    return CacheValue::Hash(fields.clone());
                }
            }
            _ => {}
        }
    }
    CacheValue::Str(raw.to_string())
}

pub(crate) fn encode_value(value: &CacheValue) -> String {
    match value {
        CacheValue::Str(s) => s.clone(),
        CacheValue::List(items) => {
            serde_json::json!({"__type": "list", "items": items}).to_string()
        }
        CacheValue::Hash(fields) => {
            serde_json::json!({"__type": "hash", "fields": fields}).to_string()
        }
    }
}

pub(crate) fn wrong_type(got: &str, _expected: &str) -> AppError {
    AppError::BadRequest(format!("WRONGTYPE Operation against a {got} key"))
}
