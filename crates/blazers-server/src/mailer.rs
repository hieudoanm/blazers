//! SMTP mailer. Configuration lives in `_settings`:
//! `smtp_host`, `smtp_port`, `smtp_user`, `smtp_from`, `app_url` (used to
//! build reset/verify links). The password is stored AES-GCM-encrypted under
//! `smtp_pass` using the server's secrets key.
//!
//! If SMTP is not configured, `send` logs the message and returns Ok so
//! dev flows (password reset etc.) still work — the token is also returned
//! to the caller in dev mode via the API response.

use std::sync::Arc;

use blazers_core::db;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::handlers::AppState;
use crate::secrets;

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
}

pub fn load_config(conn: &rusqlite::Connection, secrets_key: &[u8]) -> Option<SmtpConfig> {
    let host = db::get_setting(conn, "smtp_host").ok()??;
    let port = db::get_setting(conn, "smtp_port")
        .ok()??
        .parse::<u16>()
        .unwrap_or(587);
    let user = db::get_setting(conn, "smtp_user").ok()??;
    let from = db::get_setting(conn, "smtp_from")
        .ok()?
        .unwrap_or_else(|| user.clone());
    let pass_enc = db::get_setting(conn, "smtp_pass").ok()??;
    let pass = secrets::decrypt_secret(secrets_key, &pass_enc).unwrap_or_default();
    Some(SmtpConfig {
        host,
        port,
        user,
        pass,
        from,
    })
}

pub fn is_configured(conn: &rusqlite::Connection) -> bool {
    db::get_setting(conn, "smtp_host").ok().flatten().is_some()
}

/// Encrypts and stores the SMTP password.
pub fn set_password(
    conn: &rusqlite::Connection,
    secrets_key: &[u8],
    password: &str,
) -> Result<(), String> {
    let enc = secrets::encrypt_secret(secrets_key, password)?;
    db::set_setting(conn, "smtp_pass", &enc).map_err(|e| e.to_string())
}

pub fn app_url(conn: &rusqlite::Connection) -> String {
    db::get_setting(conn, "app_url")
        .ok()
        .flatten()
        .unwrap_or_else(|| "http://localhost:8080".into())
}

/// Sends a plain-text email. When SMTP is not configured the message is
/// logged instead of sent (dev mode).
pub async fn send(
    state: &Arc<AppState>,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let conn = state.db.get().await.map_err(|e| e.to_string())?;
    let Some(cfg) = load_config(&conn, &state.secrets_key) else {
        eprintln!("[mailer] SMTP not configured; would send to {to}: {subject}\n{body}");
        return Ok(());
    };
    drop(conn);

    let from: Mailbox = cfg
        .from
        .parse()
        .map_err(|_| "invalid smtp_from address".to_string())?;
    let to: Mailbox = to
        .parse()
        .map_err(|_| format!("invalid recipient '{to}'"))?;
    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("build email: {e}"))?;

    let creds = Credentials::new(cfg.user.clone(), cfg.pass.clone());
    let mailer = SmtpTransport::starttls_relay(&cfg.host)
        .map_err(|e| format!("smtp relay: {e}"))?
        .port(cfg.port)
        .credentials(creds)
        .build();
    mailer
        .send(&email)
        .map_err(|e| format!("send email: {e}"))?;
    Ok(())
}
