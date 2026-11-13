use chrono::Utc;
use rusqlite::{Connection, params};
use serde_json::Value;
use uuid::Uuid;

use crate::models::*;

const USER_COLS: &str =
    "id, email, password, data, verified, mfa_secret, mfa_enabled, created_at, updated_at";

fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<UserRecord> {
    let data_str: String = row.get(3)?;
    Ok(UserRecord {
        id: row.get(0)?,
        email: row.get(1)?,
        password: row.get(2)?,
        data: serde_json::from_str(&data_str).unwrap_or_else(|_| serde_json::json!({})),
        verified: row.get::<_, i64>(4)? != 0,
        mfa_secret: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
        mfa_enabled: row.get::<_, i64>(6)? != 0,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

#[allow(clippy::type_complexity)]
pub fn find_user_by_email(
    conn: &Connection,
    email: &str,
) -> Result<Option<(String, String, String, String, String)>> {
    let mut stmt = conn
        .prepare("SELECT id, email, created_at, updated_at, password FROM _users WHERE email = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![email], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(r)) => Ok(Some(r)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn insert_user(conn: &Connection, id: &str, email: &str, password: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _users (id, email, password, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, email, password, now, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("email already registered".into())
        } else {
            AppError::Internal(format!("insert user: {e}"))
        }
    })?;
    Ok(())
}

pub fn get_user(conn: &Connection, id: &str) -> Result<Option<UserRecord>> {
    let mut stmt = conn
        .prepare(&format!("SELECT {USER_COLS} FROM _users WHERE id = ?1"))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], row_to_user)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(u)) => Ok(Some(u)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn update_user_data(conn: &Connection, id: &str, data: &Value) -> Result<()> {
    conn.execute(
        "UPDATE _users SET data = ?1, updated_at = ?2 WHERE id = ?3",
        params![data.to_string(), Utc::now().to_rfc3339(), id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn update_user_password(conn: &Connection, id: &str, hash: &str) -> Result<()> {
    conn.execute(
        "UPDATE _users SET password = ?1, updated_at = ?2 WHERE id = ?3",
        params![hash, Utc::now().to_rfc3339(), id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn set_user_token(
    conn: &Connection,
    id: &str,
    kind: &str,
    token: &str,
    ttl_secs: i64,
) -> Result<()> {
    let expires = (Utc::now() + chrono::Duration::seconds(ttl_secs)).to_rfc3339();
    let (token_col, exp_col) = match kind {
        "reset" => ("reset_token", "reset_expires"),
        "verify" => ("verify_token", "verify_expires"),
        _ => return Err(AppError::BadRequest("unknown token kind".into())),
    };
    conn.execute(
        &format!("UPDATE _users SET {token_col} = ?1, {exp_col} = ?2 WHERE id = ?3"),
        params![token, expires, id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

/// Finds a user by a single-use token (reset or verify). Returns the user id
/// when the token matches and has not expired; clears nothing (caller decides).
pub fn find_user_by_token(conn: &Connection, kind: &str, token: &str) -> Result<Option<String>> {
    let (token_col, exp_col) = match kind {
        "reset" => ("reset_token", "reset_expires"),
        "verify" => ("verify_token", "verify_expires"),
        _ => return Err(AppError::BadRequest("unknown token kind".into())),
    };
    let now = Utc::now().to_rfc3339();
    let sql = format!("SELECT id FROM _users WHERE {token_col} = ?1 AND {exp_col} > ?2");
    conn.query_row(&sql, params![token, now], |row| row.get(0))
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(AppError::Internal(other.to_string())),
        })
}

pub fn clear_user_token(conn: &Connection, id: &str, kind: &str) -> Result<()> {
    let (token_col, exp_col) = match kind {
        "reset" => ("reset_token", "reset_expires"),
        "verify" => ("verify_token", "verify_expires"),
        _ => return Err(AppError::BadRequest("unknown token kind".into())),
    };
    conn.execute(
        &format!("UPDATE _users SET {token_col} = NULL, {exp_col} = NULL WHERE id = ?1"),
        params![id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

/// Stores a short-lived OTP code for passwordless login.
pub fn set_user_otp(conn: &Connection, id: &str, code: &str, ttl_secs: i64) -> Result<()> {
    let expires = (Utc::now() + chrono::Duration::seconds(ttl_secs)).to_rfc3339();
    conn.execute(
        "UPDATE _users SET otp_code = ?1, otp_expires = ?2 WHERE id = ?3",
        params![code, expires, id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

/// Finds a user whose email + OTP code match and the code has not expired.
pub fn find_user_by_otp(conn: &Connection, email: &str, code: &str) -> Result<Option<String>> {
    let now = Utc::now().to_rfc3339();
    let sql = "SELECT id FROM _users WHERE email = ?1 AND otp_code = ?2 AND otp_expires > ?3";
    conn.query_row(sql, params![email, code, now], |row| row.get(0))
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(AppError::Internal(other.to_string())),
        })
}

pub fn clear_user_otp(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE _users SET otp_code = NULL, otp_expires = NULL WHERE id = ?1",
        params![id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

/// Inserts a user without a usable password (OAuth/OTP-only accounts).
/// `password` is stored empty; password login simply cannot succeed.
pub fn insert_user_no_password(conn: &Connection, id: &str, email: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _users (id, email, password, verified, created_at, updated_at) VALUES (?1, ?2, '', 1, ?3, ?3)",
        params![id, email, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("email already registered".into())
        } else {
            AppError::Internal(format!("insert user: {e}"))
        }
    })?;
    Ok(())
}

/// Stores (or clears, with `None`) the TOTP secret and enabled flag.
pub fn set_user_mfa(
    conn: &Connection,
    id: &str,
    secret: Option<&str>,
    enabled: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE _users SET mfa_secret = ?1, mfa_enabled = ?2, updated_at = ?3 WHERE id = ?4",
        params![secret, enabled as i64, Utc::now().to_rfc3339(), id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn set_user_verified(conn: &Connection, id: &str, verified: bool) -> Result<()> {
    conn.execute(
        "UPDATE _users SET verified = ?1, updated_at = ?2 WHERE id = ?3",
        params![verified as i64, Utc::now().to_rfc3339(), id],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn new_token() -> String {
    Uuid::new_v4().to_string().replace('-', "") + &Uuid::new_v4().to_string().replace('-', "")
}

/// JSON view of a user for filter/sort/rule evaluation: meta fields plus
/// custom `data` keys merged at top level (same shape as `auth_ctx`).
pub fn user_view(u: &UserRecord) -> Value {
    let mut view = serde_json::Map::new();
    view.insert("id".into(), Value::String(u.id.clone()));
    view.insert("email".into(), Value::String(u.email.clone()));
    view.insert("verified".into(), Value::Bool(u.verified));
    view.insert("created".into(), Value::String(u.created_at.clone()));
    view.insert("updated".into(), Value::String(u.updated_at.clone()));
    if let Value::Object(map) = &u.data {
        for (k, v) in map {
            view.insert(k.clone(), v.clone());
        }
    }
    Value::Object(view)
}

/// Lists users with optional filter expression and sort spec — same
/// semantics as `list_records_query` but over `_users`.
pub fn list_users(
    conn: &Connection,
    page: i64,
    per_page: i64,
    filter: Option<&str>,
    sort: Option<&str>,
    auth: &Value,
) -> Result<(Vec<UserRecord>, i64)> {
    let compiled = crate::filter::compile(filter.unwrap_or(""))
        .map_err(|e| AppError::BadRequest(format!("invalid filter: {e}")))?;

    let mut stmt = conn
        .prepare(&format!(
            "SELECT {USER_COLS} FROM _users ORDER BY created_at DESC"
        ))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], row_to_user)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut matched: Vec<UserRecord> = Vec::new();
    for row in rows {
        let u = row.map_err(|e| AppError::Internal(e.to_string()))?;
        if compiled.matches(&user_view(&u), auth) {
            matched.push(u);
        }
    }

    if let Some(spec) = sort.filter(|s| !s.trim().is_empty()) {
        let keys: Vec<(String, bool)> = spec
            .split(',')
            .map(|part| {
                let p = part.trim();
                if let Some(name) = p.strip_prefix('-') {
                    (name.to_string(), true)
                } else {
                    (p.to_string(), false)
                }
            })
            .collect();
        matched.sort_by(|a, b| {
            for (field, desc) in &keys {
                let va = resolve_user_path(&user_view(a), field);
                let vb = resolve_user_path(&user_view(b), field);
                let (ra, rb) = (crate::filter::sort_rank(&va), crate::filter::sort_rank(&vb));
                let ord = if *desc {
                    ra.cmp(&rb).reverse()
                } else {
                    ra.cmp(&rb)
                };
                let ord = match (va.is_null(), vb.is_null()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => ord,
                };
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        });
    }

    let total = matched.len() as i64;
    let offset = ((page - 1) * per_page) as usize;
    let end = (offset + per_page as usize).min(matched.len());
    let users = if offset >= matched.len() {
        Vec::new()
    } else {
        matched[offset..end].to_vec()
    };
    Ok((users, total))
}

fn resolve_user_path(view: &Value, path: &str) -> Value {
    let mut cur = view.clone();
    for part in path.split('.') {
        cur = match cur.get(part) {
            Some(v) => v.clone(),
            None => return Value::Null,
        };
    }
    cur
}
