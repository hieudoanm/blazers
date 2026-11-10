use crate::db::validate::validate_field;
use crate::db::*;
use crate::models::*;
use serde_json::json;

#[test]
fn test_validate_field_string_valid() {
    assert_eq!(validate_field(&json!("hello"), "string", "name"), None);
}

#[test]
fn test_validate_field_string_invalid() {
    let err = validate_field(&json!(42), "string", "name");
    assert_eq!(err, Some("field 'name' must be a string".to_string()));
}

#[test]
fn test_validate_field_number_valid() {
    assert_eq!(validate_field(&json!(4.2), "number", "x"), None);
}

#[test]
fn test_validate_field_number_invalid() {
    let err = validate_field(&json!("text"), "number", "x");
    assert_eq!(err, Some("field 'x' must be a number".to_string()));
}

#[test]
fn test_validate_field_integer_valid() {
    assert_eq!(validate_field(&json!(42), "integer", "n"), None);
}

#[test]
fn test_validate_field_integer_float() {
    let err = validate_field(&json!(3.5), "integer", "n");
    assert_eq!(err, Some("field 'n' must be an integer".to_string()));
}

#[test]
fn test_validate_field_integer_non_number() {
    let err = validate_field(&json!("bad"), "integer", "n");
    assert_eq!(err, Some("field 'n' must be an integer".to_string()));
}

#[test]
fn test_validate_field_boolean_valid() {
    assert_eq!(validate_field(&json!(true), "boolean", "b"), None);
    assert_eq!(validate_field(&json!(false), "boolean", "b"), None);
}

#[test]
fn test_validate_field_boolean_invalid() {
    let err = validate_field(&json!("yes"), "boolean", "b");
    assert_eq!(err, Some("field 'b' must be a boolean".to_string()));
}

#[test]
fn test_validate_field_array_valid() {
    assert_eq!(validate_field(&json!([1, 2, 3]), "array", "a"), None);
}

#[test]
fn test_validate_field_array_invalid() {
    let err = validate_field(&json!("not array"), "array", "a");
    assert_eq!(err, Some("field 'a' must be an array".to_string()));
}

#[test]
fn test_validate_field_object_valid() {
    assert_eq!(validate_field(&json!({"k": "v"}), "object", "o"), None);
}

#[test]
fn test_validate_field_object_invalid() {
    let err = validate_field(&json!("not object"), "object", "o");
    assert_eq!(err, Some("field 'o' must be an object".to_string()));
}

#[test]
fn test_validate_field_email_valid() {
    assert_eq!(
        validate_field(&json!("user@example.com"), "email", "e"),
        None
    );
}

#[test]
fn test_validate_field_email_missing_at() {
    let err = validate_field(&json!("userexample.com"), "email", "e");
    assert_eq!(err, Some("field 'e' must be a valid email".to_string()));
}

#[test]
fn test_validate_field_email_non_string() {
    let err = validate_field(&json!(42), "email", "e");
    assert_eq!(err, Some("field 'e' must be a valid email".to_string()));
}

#[test]
fn test_validate_field_url_valid_http() {
    assert_eq!(
        validate_field(&json!("http://example.com"), "url", "u"),
        None
    );
}

#[test]
fn test_validate_field_url_valid_https() {
    assert_eq!(
        validate_field(&json!("https://example.com"), "url", "u"),
        None
    );
}

#[test]
fn test_validate_field_url_no_protocol() {
    let err = validate_field(&json!("example.com"), "url", "u");
    assert_eq!(err, Some("field 'u' must be a valid URL".to_string()));
}

#[test]
fn test_validate_field_unknown_type() {
    assert_eq!(validate_field(&json!("anything"), "custom_type", "c"), None);
}

#[test]
fn test_validate_data_empty_schema() {
    assert_eq!(validate_data(&json!({"x": 1}), "").ok(), Some(()));
    assert_eq!(validate_data(&json!({"x": 1}), "{}").ok(), Some(()));
}

#[test]
fn test_validate_data_all_valid() {
    let data = json!({"name": "Alice", "age": 30, "active": true});
    let schema = r#"{"name": "string", "age": "integer", "active": "boolean"}"#;
    assert!(validate_data(&data, schema).is_ok());
}

#[test]
fn test_validate_data_missing_required() {
    let data = json!({"name": "Alice"});
    let schema = r#"{"name": "string", "age": "integer"}"#;
    let err = validate_data(&data, schema).unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    assert!(format!("{:?}", err).contains("age"));
}

#[test]
fn test_validate_data_optional_missing() {
    let data = json!({"name": "Alice"});
    let schema = r#"{"name": "string", "age?": "integer"}"#;
    assert!(validate_data(&data, schema).is_ok());
}

#[test]
fn test_validate_data_optional_null() {
    let data = json!({"name": "Alice", "age": null});
    let schema = r#"{"name": "string", "age?": "integer"}"#;
    assert!(validate_data(&data, schema).is_ok());
}

#[test]
fn test_validate_data_type_mismatch() {
    let data = json!({"name": 42});
    let schema = r#"{"name": "string"}"#;
    let err = validate_data(&data, schema).unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    assert!(format!("{:?}", err).contains("must be a string"));
}

#[test]
fn test_validate_data_multiple_errors() {
    let data = json!({"a": 1, "b": "not bool", "c": [1,2,3]});
    let schema = r#"{"a": "string", "b": "boolean", "c": "number"}"#;
    let err = validate_data(&data, schema).unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let msg = format!("{:?}", err);
    assert!(msg.contains("a") && msg.contains("b") && msg.contains("c"));
}

// --- data_dir ---
