//! OAuth2 sign-in (PocketBase `authWithOAuth2` parity):
//! `GET /api/auth/oauth/{provider}` redirects to the provider's authorize
//! page; `GET /api/auth/oauth/{provider}/callback` exchanges the code,
//! finds-or-creates the user by email and returns a Blazers JWT.
//!
//! Configuration lives in `oauth_config.rs` — `app_url` must be the public
//! base URL so providers can redirect back to `/api/auth/oauth/*/callback`.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use super::oauth_config::*;
use super::*;
use crate::auth;

/// GET /api/auth/oauth/{provider} — 302 to the provider authorize page.
pub async fn oauth_start(
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
) -> std::result::Result<Response, AppError> {
    let p = provider(&name)
        .ok_or_else(|| AppError::NotFound(format!("unknown oauth provider '{name}'")))?;
    let (client_id, _) = client_config(&state, &name).await?;
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let base = crate::mailer::app_url(&conn);
    drop(conn);
    let redirect_uri = format!("{base}/api/auth/oauth/{name}/callback");
    let state_token = make_state(&name)?;
    let url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        p.authorize,
        url_encode(&client_id),
        url_encode(&redirect_uri),
        url_encode(scope_for(&name)),
        url_encode(&state_token),
    );
    Ok(Redirect::temporary(&url).into_response())
}

#[derive(Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

async fn exchange_code(
    state: &AppState,
    p: &Provider,
    provider: &str,
    code: &str,
    redirect_uri: &str,
) -> std::result::Result<String, AppError> {
    let (client_id, client_secret) = client_config(state, provider).await?;
    let body = format!(
        "grant_type=authorization_code&code={}&client_id={}&client_secret={}&redirect_uri={}",
        url_encode(code),
        url_encode(&client_id),
        url_encode(&client_secret),
        url_encode(redirect_uri),
    );
    let resp = state
        .http_client
        .post(p.token)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("oauth token exchange: {e}")))?;
    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("oauth token response: {e}")))?;
    body.get("access_token")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AppError::BadRequest("oauth token exchange failed".into()))
}

async fn fetch_email(
    state: &AppState,
    p: &Provider,
    provider: &str,
    access_token: &str,
) -> std::result::Result<(String, String), AppError> {
    let mut req = state
        .http_client
        .get(p.userinfo)
        .bearer_auth(access_token)
        .header("User-Agent", "blazers");
    if provider == "github" {
        req = req.header("Accept", "application/vnd.github+json");
    }
    let profile: Value = req
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("oauth userinfo: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("oauth userinfo parse: {e}")))?;

    let mut email = profile
        .get("email")
        .and_then(Value::as_str)
        .map(str::to_string);
    // GitHub hides the email when private — fall back to /user/emails.
    if email.is_none() && provider == "github" {
        let emails: Value = state
            .http_client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "blazers")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("oauth emails: {e}")))?
            .json()
            .await
            .unwrap_or(Value::Null);
        email = emails
            .as_array()
            .and_then(|arr| {
                arr.iter()
                    .find(|e| e["primary"] == true && e["verified"] == true)
                    .or_else(|| arr.first())
            })
            .and_then(|e| e["email"].as_str())
            .map(str::to_string);
    }
    let subject = profile
        .get("sub")
        .or_else(|| profile.get("id"))
        .map(|v| v.to_string())
        .unwrap_or_default();
    email
        .filter(|e| e.contains('@'))
        .map(|e| (e, subject))
        .ok_or_else(|| AppError::BadRequest("oauth provider did not return an email".into()))
}

/// GET /api/auth/oauth/{provider}/callback?code=&state= — finishes sign-in.
pub async fn oauth_callback(
    State(state): State<std::sync::Arc<AppState>>,
    Path(name): Path<String>,
    Query(q): Query<OAuthCallback>,
) -> std::result::Result<Json<blazers_core::models::LoginResponse>, AppError> {
    let p = provider(&name)
        .ok_or_else(|| AppError::NotFound(format!("unknown oauth provider '{name}'")))?;
    if let Some(err) = q.error {
        return Err(AppError::BadRequest(format!("oauth error: {err}")));
    }
    let code = q
        .code
        .filter(|c| !c.is_empty())
        .ok_or_else(|| AppError::BadRequest("missing oauth code".into()))?;
    let state_token = q
        .state
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("missing oauth state".into()))?;
    check_state(&state_token, &name)?;

    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let base = crate::mailer::app_url(&conn);
    drop(conn);
    let redirect_uri = format!("{base}/api/auth/oauth/{name}/callback");

    let access_token = exchange_code(&state, &p, &name, &code, &redirect_uri).await?;
    let (email, subject) = fetch_email(&state, &p, &name, &access_token).await?;

    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user_id = match db::find_user_by_email(&conn, &email)? {
        Some((id, _, _, _, _)) => id,
        None => {
            let id = Uuid::new_v4().to_string().replace('-', "");
            db::insert_user_no_password(&conn, &id, &email)?;
            // Record which provider identity is linked.
            if let Some(mut u) = db::get_user(&conn, &id)?
                && let Value::Object(ref mut map) = u.data
            {
                map.insert(format!("oauth_{name}"), Value::String(subject));
                let _ = db::update_user_data(&conn, &id, &u.data);
            }
            id
        }
    };
    let user = db::get_user(&conn, &user_id)?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let token = auth::issue_token(&user.id, &user.email)?;
    Ok(Json(blazers_core::models::LoginResponse {
        user: auth::user_response(&user),
        token,
    }))
}
