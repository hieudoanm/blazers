use super::*;
use serde_json::json;

fn rec() -> Value {
    json!({
        "id": "r1",
        "created": "2024-01-01",
        "updated": "2024-01-02",
        "title": "Hello World",
        "status": "published",
        "views": 42,
        "featured": true,
        "author": {"id": "u1", "role": "admin"},
        "tags": ["rust", "db"]
    })
}

fn auth() -> Value {
    json!({"id": "u1", "email": "a@b.c", "role": "admin"})
}

#[test]
fn equality_and_inequality() {
    assert!(eval("status = 'published'", &rec(), &auth()).unwrap());
    assert!(!eval("status = 'draft'", &rec(), &auth()).unwrap());
    assert!(eval("status != 'draft'", &rec(), &auth()).unwrap());
    assert!(eval("views = 42", &rec(), &auth()).unwrap());
    assert!(eval("featured = true", &rec(), &auth()).unwrap());
    assert!(eval("missing = null", &rec(), &auth()).unwrap());
}

#[test]
fn boolean_operators_and_parens() {
    assert!(eval("status = 'published' && views > 10", &rec(), &auth()).unwrap());
    assert!(eval("status = 'draft' || views > 10", &rec(), &auth()).unwrap());
    assert!(!eval("status = 'draft' && views > 10", &rec(), &auth()).unwrap());
    assert!(eval("!(status = 'draft')", &rec(), &auth()).unwrap());
    assert!(
        eval(
            "(status = 'draft' || status = 'published') && featured",
            &rec(),
            &auth()
        )
        .unwrap()
    );
}

#[test]
fn comparisons() {
    assert!(eval("views >= 42", &rec(), &auth()).unwrap());
    assert!(eval("views < 100", &rec(), &auth()).unwrap());
    assert!(eval("title > 'A'", &rec(), &auth()).unwrap());
}

#[test]
fn contains_operator() {
    assert!(eval("title ~ 'world'", &rec(), &auth()).unwrap());
    assert!(!eval("title ~ 'xyz'", &rec(), &auth()).unwrap());
    assert!(eval("title !~ 'xyz'", &rec(), &auth()).unwrap());
}

#[test]
fn nested_paths() {
    assert!(eval("author.role = 'admin'", &rec(), &auth()).unwrap());
    assert!(eval("author.id = 'u1'", &rec(), &auth()).unwrap());
}

#[test]
fn request_auth_paths() {
    assert!(eval("@request.auth.id = 'u1'", &rec(), &auth()).unwrap());
    assert!(eval("@request.auth.role = 'admin'", &rec(), &auth()).unwrap());
    assert!(!eval("@request.auth.role = 'viewer'", &rec(), &auth()).unwrap());
    assert!(eval("@request.auth.missing = null", &rec(), &auth()).unwrap());
}

#[test]
fn truthy_operand() {
    assert!(eval("featured", &rec(), &auth()).unwrap());
    assert!(!eval("missing_field", &rec(), &auth()).unwrap());
    assert!(!eval("views = 0", &rec(), &auth()).unwrap());
}

#[test]
fn invalid_expressions_error() {
    assert!(eval("status === 'x'", &rec(), &auth()).is_err());
    assert!(eval("(status = 'x'", &rec(), &auth()).is_err());
    assert!(eval("status = 'unterminated", &rec(), &auth()).is_err());
}

#[test]
fn empty_filter_matches_all() {
    assert!(eval("", &rec(), &auth()).unwrap());
    assert!(eval("   ", &rec(), &auth()).unwrap());
}

#[test]
fn compile_reuse() {
    let f = compile("status = 'published' && views > 10").unwrap();
    assert!(f.matches(&rec(), &auth()));
    let bad = rec();
    let mut other = bad.clone();
    other["status"] = json!("draft");
    assert!(!f.matches(&other, &auth()));
}

#[test]
fn sort_rank_orders() {
    assert!(sort_rank(&json!(5)) < sort_rank(&json!(10)));
    assert!(sort_rank(&json!("abc")) > sort_rank(&json!(9)));
    assert!(sort_rank(&Value::Null) > sort_rank(&json!(true)));
}
