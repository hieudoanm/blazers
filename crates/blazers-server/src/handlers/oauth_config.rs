//! OAuth2 provider table, client config resolution and the signed `state`
//! token used to protect the authorize→callback round-trip.

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Provider {
    pub authorize: &'static str,
    pub token: &'static str,
    pub userinfo: &'static str,
}

pub(crate) fn provider(name: &str) -> Option<Provider> {
    Some(match name {
        "google" => Provider {
            authorize: "https://accounts.google.com/o/oauth2/v2/auth",
            token: "https://oauth2.googleapis.com/token",
            userinfo: "https://openidconnect.googleapis.com/v1/userinfo",
        },
        "github" => Provider {
            authorize: "https://github.com/login/oauth/authorize",
            token: "https://github.com/login/oauth/access_token",
            userinfo: "https://api.github.com/user",
        },
        "gitlab" => Provider {
            authorize: "https://gitlab.com/oauth/authorize",
            token: "https://gitlab.com/oauth/token",
            userinfo: "https://gitlab.com/api/v4/user",
        },
        "discord" => Provider {
            authorize: "https://discord.com/oauth2/authorize",
            token: "https://discord.com/api/oauth2/token",
            userinfo: "https://discord.com/api/users/@me",
        },
        _ => return None,
    })
}

pub(crate) fn scope_for(provider: &str) -> &'static str {
    match provider {
        "google" => "openid email profile",
        "github" => "read:user user:email",
        "gitlab" => "read_user",
        "discord" => "identify email",
        _ => "",
    }
}

fn env_key(provider: &str, suffix: &str) -> String {
    format!("OAUTH_{}_{}", provider.to_uppercase(), suffix)
}

/// Client id/secret: `_settings` first, env vars as fallback.
pub(crate) async fn client_config(
    state: &AppState,
    provider: &str,
) -> std::result::Result<(String, String), AppError> {
    let conn = state
        .db
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let get = |key: &str| {
        db::get_setting(&conn, key)
            .ok()
            .flatten()
            .filter(|s| !s.is_empty())
    };
    let missing = || AppError::BadRequest(format!("oauth provider '{provider}' is not configured"));
    let id = get(&format!("oauth_{provider}_client_id"))
        .or_else(|| std::env::var(env_key(provider, "CLIENT_ID")).ok())
        .ok_or_else(missing)?;
    let secret = get(&format!("oauth_{provider}_client_secret"))
        .or_else(|| std::env::var(env_key(provider, "CLIENT_SECRET")).ok())
        .ok_or_else(missing)?;
    Ok((id, secret))
}

pub(crate) fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OAuthState {
    purpose: String,
    provider: String,
    nonce: String,
    exp: usize,
    iat: usize,
}

fn state_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".into())
}

pub(crate) fn make_state(provider: &str) -> Result<String> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = OAuthState {
        purpose: "oauth".into(),
        provider: provider.into(),
        nonce: Uuid::new_v4().to_string(),
        exp: now + 600,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state_secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("state: {e}")))
}

pub(crate) fn check_state(token: &str, provider: &str) -> Result<()> {
    let claims = decode::<OAuthState>(
        token,
        &DecodingKey::from_secret(state_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::BadRequest("invalid oauth state".into()))?
    .claims;
    if claims.purpose != "oauth" || claims.provider != provider {
        return Err(AppError::BadRequest("invalid oauth state".into()));
    }
    Ok(())
}
