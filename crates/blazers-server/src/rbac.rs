use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use rusqlite::{Connection, params};

use crate::handlers::AppState;
use blazers_core::db;
use blazers_core::models::*;

pub fn require_role(
    conn: &Connection,
    user_id: &str,
    collection: &str,
    required: &str,
) -> std::result::Result<(), AppError> {
    let mut stmt = conn
        .prepare("SELECT role FROM _permissions WHERE user_id = ?1 AND (collection = ?2 OR collection = '*')")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let roles: Vec<String> = stmt
        .query_map(params![user_id, collection], |row| row.get(0))
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    for role in &roles {
        if role == "admin" || role == required {
            return Ok(());
        }
    }
    if collection != "*" {
        let mut stmt = conn
            .prepare("SELECT role FROM _permissions WHERE user_id = ?1 AND collection = '*'")
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let global_roles: Vec<String> = stmt
            .query_map(params![user_id], |row| row.get(0))
            .map_err(|e| AppError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        for role in &global_roles {
            if role == "admin" || role == required {
                return Ok(());
            }
        }
    }
    Err(AppError::Forbidden("insufficient permissions".into()))
}

/// Returns true when the user holds the `admin` role on `collection` (or `*`).
pub fn is_admin(
    conn: &Connection,
    user_id: &str,
    collection: &str,
) -> std::result::Result<bool, AppError> {
    let mut stmt = conn
        .prepare("SELECT role FROM _permissions WHERE user_id = ?1 AND (collection = ?2 OR collection = '*')")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let roles: Vec<String> = stmt
        .query_map(params![user_id, collection], |row| row.get(0))
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(roles.iter().any(|r| r == "admin"))
}

/// Builds the `@request.auth` JSON context used by record rules.
pub fn auth_ctx(conn: &Connection, user_id: &str, email: &str) -> serde_json::Value {
    let user = db::get_user(conn, user_id).ok().flatten();
    let mut data = serde_json::Map::new();
    data.insert("id".into(), serde_json::json!(user_id));
    data.insert("email".into(), serde_json::json!(email));
    if let Some(u) = user {
        data.insert("verified".into(), serde_json::json!(u.verified));
        if let serde_json::Value::Object(map) = &u.data {
            for (k, v) in map {
                data.insert(k.clone(), v.clone());
            }
        }
    }
    serde_json::Value::Object(data)
}

/// Evaluates one PocketBase-style rule expression against a record *view*
/// (the merged `{id, created, updated, ...data}` JSON). Empty rule = allow;
/// admins bypass; a rule that fails to parse denies.
pub fn check_rule_view(
    conn: &Connection,
    collection: &blazers_core::models::Collection,
    rule: &str,
    record: &serde_json::Value,
    auth: &serde_json::Value,
    user_id: &str,
) -> std::result::Result<(), AppError> {
    if rule.trim().is_empty() || is_admin(conn, user_id, &collection.name)? {
        return Ok(());
    }
    match blazers_core::filter::eval(rule, record, auth) {
        Ok(true) => Ok(()),
        Ok(false) => Err(AppError::Forbidden("record rule denied access".into())),
        Err(e) => Err(AppError::Internal(format!("record rule error: {e}"))),
    }
}

/// View rule for `view`/`update`/`delete` on a single fetched record.
pub fn check_record_rule(
    conn: &Connection,
    collection: &blazers_core::models::Collection,
    rule_field: RuleField,
    record: &blazers_core::models::Record,
    auth: &serde_json::Value,
    user_id: &str,
) -> std::result::Result<(), AppError> {
    let rule = match rule_field {
        RuleField::List => &collection.list_rule,
        RuleField::View => &collection.view_rule,
        RuleField::Create => &collection.create_rule,
        RuleField::Update => &collection.update_rule,
        RuleField::Delete => &collection.delete_rule,
    };
    check_rule_view(
        conn,
        collection,
        rule,
        &blazers_core::db::record_view(record),
        auth,
        user_id,
    )
}

pub enum RuleField {
    List,
    View,
    Create,
    Update,
    Delete,
}

/// Header-based convenience wrapper around [`require_role`].
pub async fn require_role_pub(
    state: &std::sync::Arc<crate::handlers::AppState>,
    headers: &HeaderMap,
    collection: &str,
    required: &str,
) -> std::result::Result<(), AppError> {
    let claims = crate::handlers::extract_claims(headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    require_role(&conn, &claims.user_id, collection, required)
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub user_id: String,
    pub collection: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PermissionRequest {
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub collection: String,
    #[serde(default)]
    pub role: String,
}

pub(crate) mod handlers;
pub use handlers::*;

#[cfg(test)]
mod tests;
