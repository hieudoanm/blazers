use super::*;

#[test]
fn permission_round_trip() {
    let p = Permission {
        id: "p1".into(),
        user_id: "u1".into(),
        collection: "c1".into(),
        role: "admin".into(),
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2025-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&p).unwrap();
    let d: Permission = serde_json::from_str(&json).unwrap();
    assert_eq!(p.id, d.id);
    assert_eq!(p.user_id, d.user_id);
    assert_eq!(p.collection, d.collection);
    assert_eq!(p.role, d.role);
}

#[test]
fn permission_request_defaults_empty() {
    let req: PermissionRequest = serde_json::from_str("{}").unwrap();
    assert_eq!(req.user_id, "");
    assert_eq!(req.collection, "");
    assert_eq!(req.role, "");
}

#[test]
fn test_require_role_admin_global() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', '*', 'admin')",
            [],
        )
        .unwrap();
    assert!(require_role(&conn, "u1", "any_collection", "viewer").is_ok());
}

#[test]
fn test_require_role_exact_match() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'docs', 'editor')",
            [],
        )
        .unwrap();
    assert!(require_role(&conn, "u1", "docs", "editor").is_ok());
}

#[test]
fn test_require_role_admin_exact() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'docs', 'admin')",
            [],
        )
        .unwrap();
    assert!(require_role(&conn, "u1", "docs", "viewer").is_ok());
}

#[test]
fn test_require_role_insufficient() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'other', 'viewer')",
            [],
        )
        .unwrap();
    let result = require_role(&conn, "u1", "test", "viewer");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[test]
fn test_require_role_no_permissions() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    let result = require_role(&conn, "u1", "test", "viewer");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[test]
fn test_require_role_global_viewer() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', '*', 'viewer')",
            [],
        )
        .unwrap();
    assert!(require_role(&conn, "u1", "some_collection", "viewer").is_ok());
}

#[test]
fn test_require_role_different_user() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', '*', 'admin')",
            [],
        )
        .unwrap();
    let result = require_role(&conn, "u2", "test", "viewer");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[test]
fn test_find_permission_by_id_found() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    create_perms_table(&conn);
    conn.execute(
            "INSERT INTO _permissions (id, user_id, collection, role) VALUES ('p1', 'u1', 'c1', 'editor')",
            [],
        )
        .unwrap();
    let found = find_permission_by_id(&conn, "p1").unwrap().unwrap();
    assert_eq!(found.user_id, "u1");
    assert_eq!(found.role, "editor");
    assert_eq!(found.collection, "c1");
}
