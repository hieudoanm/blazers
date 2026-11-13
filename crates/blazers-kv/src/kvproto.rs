//! Redis-style inline protocol over TCP — the `blazers kv` listener.
//!
//! Clients send one command per line (`SET key value`, `GET key`, …) and get a
//! single reply line, behaviour-compatible with the KeVIN reference server.
//! Values in typed commands are space-free tokens; `SET` preserves spaces in
//! the value like the reference. When `BLAZERS_KV_PASSWORD` is set, clients
//! must `AUTH` before any other command.

use std::sync::Arc;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::cache::CacheStore;

/// handle_line parses one protocol line and returns the reply plus whether a
/// reply should be sent. Empty/whitespace lines are ignored, matching the
/// reference implementation.
pub fn handle_line(
    line: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
    authed: &mut bool,
    require_auth: bool,
) -> (String, bool) {
    let line = line.trim_end_matches(['\r', '\n']);
    if line.trim().is_empty() {
        return (String::new(), false);
    }

    let (cmd, rest) = split_token(line);
    let cmd = cmd.to_ascii_uppercase();

    if cmd == "AUTH" {
        return handle_auth(rest, authed, require_auth);
    }
    if require_auth && !*authed {
        return ("NOAUTH Authentication required.\n".to_string(), true);
    }

    let result = dispatch(&cmd, rest, cache, conn);
    match result {
        Ok(reply) => (reply, true),
        Err(err) => (format!("ERR {err}\n"), true),
    }
}

mod basic;
mod commands;
mod hashes;
mod lists;
mod ops;

use commands::{dispatch, handle_auth};

pub fn split_token(s: &str) -> (&str, &str) {
    let bytes = s.as_bytes();
    let mut start = 0;
    while start < bytes.len() && bytes[start] == b' ' {
        start += 1;
    }
    let mut end = start;
    while end < bytes.len() && bytes[end] != b' ' {
        end += 1;
    }
    (&s[start..end], &s[end..])
}

/// Returns every space-delimited token of `s`.
pub fn split_all(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = s;
    loop {
        let (token, after) = split_token(rest);
        if token.is_empty() {
            break;
        }
        tokens.push(token.to_string());
        rest = after;
    }
    tokens
}

/// Consumes exactly two space-delimited tokens of `s`.
pub fn split_two(s: &str) -> Option<(&str, &str)> {
    let (first, rest) = split_token(s);
    if first.is_empty() {
        return None;
    }
    let (second, rest) = split_token(rest);
    if second.is_empty() || !rest.trim().is_empty() {
        return None;
    }
    Some((first, second))
}

/// serve_kv runs the inline-protocol listener until the process exits.
/// Each client gets its own task; database access goes through the shared
/// pool, and `BLAZERS_KV_PASSWORD` optionally gates every command behind AUTH.
pub async fn serve_kv(bind: &str, port: u16) -> std::io::Result<()> {
    let data_dir = blazers_core::db::data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let pool: deadpool::managed::Pool<blazers_core::db::ConnectionManager> =
        deadpool::managed::Pool::builder(blazers_core::db::ConnectionManager {
            path: data_dir.join("data.db"),
        })
        .build()
        .map_err(|e| std::io::Error::other(format!("db pool: {e}")))?;
    let cache = Arc::new(CacheStore::new());
    {
        let conn = pool
            .get()
            .await
            .map_err(|e| std::io::Error::other(format!("db connect: {e}")))?;
        blazers_core::db::migrate_db(&conn)
            .map_err(|e| std::io::Error::other(format!("migrate: {e}")))?;
        cache.load_from_db(&conn);
    }
    let require_auth = std::env::var("BLAZERS_KV_PASSWORD").is_ok();

    let listener = TcpListener::bind((bind, port))
        .await
        .map_err(|e| std::io::Error::other(format!("bind {bind}:{port}: {e}")))?;
    eprintln!("blazers kv listening on {bind}:{port}");
    loop {
        let (stream, peer) = tokio::select! {
            accepted = listener.accept() => accepted
                .map_err(|e| std::io::Error::other(format!("accept: {e}")))?,
            _ = tokio::signal::ctrl_c() => {
                eprintln!("blazers kv shutting down");
                return Ok(());
            }
        };
        let pool = pool.clone();
        let cache = Arc::clone(&cache);
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, pool, cache, require_auth).await {
                eprintln!("kv client {peer}: {err}");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    pool: deadpool::managed::Pool<blazers_core::db::ConnectionManager>,
    cache: Arc<CacheStore>,
    require_auth: bool,
) -> std::io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let mut authed = false;
    while let Some(line) = lines.next_line().await? {
        let conn = match pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                let _ = writer
                    .write_all(format!("ERR database unavailable: {err}\n").as_bytes())
                    .await;
                continue;
            }
        };
        let (reply, should_reply) = handle_line(&line, &cache, &conn, &mut authed, require_auth);
        drop(conn);
        if should_reply {
            writer.write_all(reply.as_bytes()).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
