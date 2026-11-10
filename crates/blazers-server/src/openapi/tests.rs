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
