//! Managed `users` auth collection: admin listing, per-user fetch and the
//! five PocketBase-style rule expressions stored under the `users_rules`
//! setting. Non-admin callers are filtered by `list_rule`/`view_rule`;
//! an empty rule means "admins/self only".

use super::*;

const RULES_KEY: &str = "users_rules";

fn load_rules(conn: &rusqlite::Connection) -> UserRules {
    db::get_setting(conn, RULES_KEY)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_rules(conn: &rusqlite::Connection, rules: &UserRules) -> std::result::Result<(), AppError> {
    let json = serde_json::to_string(rules)
        .map_err(|e| AppError::Internal(format!("serialize rules: {e}")))?;
    db::set_setting(conn, RULES_KEY, &json).map_err(|e| AppError::Internal(e.to_string()))
}

fn eval_rule(rule: &str, view: &Value, auth: &Value) -> bool {
    rule.trim().is_empty() || blazers_core::filter::eval(rule, view, auth).unwrap_or(false)
}

/// GET /api/auth/users/rules — current users-collection rules (admin).
pub async fn get_user_rules(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
) -> std::result::Result<Json<UserRules>, AppError> {
    crate::rbac::require_role_pub(&state, &headers, "*", "admin").await?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(load_rules(&conn)))
}

/// PATCH /api/auth/users/rules — set users-collection rules (admin).
pub async fn update_user_rules(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<UserRules>,
) -> std::result::Result<Json<UserRules>, AppError> {
    crate::rbac::require_role_pub(&state, &headers, "*", "admin").await?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    // Validate expressions parse before persisting.
    for rule in [
        &req.list_rule,
        &req.view_rule,
        &req.create_rule,
        &req.update_rule,
        &req.delete_rule,
    ] {
        if !rule.trim().is_empty() {
            blazers_core::filter::compile(rule)
                .map_err(|e| AppError::BadRequest(format!("invalid rule: {e}")))?;
        }
    }
    save_rules(&conn, &req)?;
    Ok(Json(req))
}

#[derive(serde::Deserialize)]
pub struct UserListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub filter: Option<String>,
    pub sort: Option<String>,
}

/// GET /api/auth/users — list users. Admins see all; other callers are
/// restricted to their own record plus anything `list_rule` allows.
pub async fn list_auth_users(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Query(q): Query<UserListQuery>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let admin = rbac::is_admin(&conn, &claims.user_id, "*")?;
    let rules = load_rules(&conn);
    let auth = rbac::auth_ctx(&conn, &claims.user_id, &claims.email);

    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);

    // Non-admins: AND the client filter with list_rule; empty rule = self only.
    let effective = if admin {
        q.filter.clone()
    } else {
        let rule = if rules.list_rule.trim().is_empty() {
            format!("id = '{}'", claims.user_id)
        } else {
            rules.list_rule.clone()
        };
        match q.filter.as_deref().map(str::trim).filter(|f| !f.is_empty()) {
            Some(f) => Some(format!("({f}) && ({rule})")),
            None => Some(rule),
        }
    };

    let (users, total) = db::list_users(
        &conn,
        page,
        per_page,
        effective.as_deref(),
        q.sort.as_deref(),
        &auth,
    )?;
    let items: Vec<Value> = users.iter().map(user_json).collect();
    Ok(Json(serde_json::json!({
        "users": items,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": (total + per_page - 1) / per_page.max(1),
    })))
}

/// GET /api/auth/users/{id} — fetch one user. Self and admins always;
/// others need `view_rule` to match.
pub async fn get_auth_user(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Value>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user =
        db::get_user(&conn, &id)?.ok_or_else(|| AppError::NotFound("user not found".into()))?;

    if claims.user_id != id {
        let admin = rbac::is_admin(&conn, &claims.user_id, "*")?;
        if !admin {
            let rules = load_rules(&conn);
            let auth = rbac::auth_ctx(&conn, &claims.user_id, &claims.email);
            if !eval_rule(&rules.view_rule, &db::user_view(&user), &auth) {
                return Err(AppError::Forbidden("user rule denied access".into()));
            }
        }
    }
    Ok(Json(user_json(&user)))
}

/// POST /api/auth/users/{id}/impersonate — admin-only; returns a user-scoped
/// JWT for the target account (PocketBase `impersonate` parity).
pub async fn impersonate_user(
    headers: HeaderMap,
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> std::result::Result<Json<blazers_core::models::LoginResponse>, AppError> {
    let claims = extract_claims(&headers)?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !rbac::is_admin(&conn, &claims.user_id, "*")? {
        return Err(AppError::Forbidden("admin role required".into()));
    }
    let user =
        db::get_user(&conn, &id)?.ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let token = crate::auth::issue_token(&user.id, &user.email)?;
    Ok(Json(blazers_core::models::LoginResponse {
        user: crate::auth::user_response(&user),
        token,
    }))
}
