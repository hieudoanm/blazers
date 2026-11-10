use super::*;

pub(super) fn handle_expire(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let Some((key, seconds)) = split_two(rest) else {
        return Err("usage: EXPIRE key seconds".to_string());
    };
    let ttl: i64 = seconds
        .parse()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| "invalid expire time".to_string())?;
    let n = if cache
        .expire(conn, key, ttl)
        .map_err(|e| e.to_string())?
        .is_some()
    {
        "1"
    } else {
        "0"
    };
    Ok(format!("{n}\n"))
}

pub(super) fn handle_ttl(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    if key.is_empty() || !after.trim().is_empty() {
        return Err("usage: TTL key".to_string());
    }
    let value = match cache.ttl(conn, key).map_err(|e| e.to_string())? {
        None => -2,
        Some(-1) => -1,
        Some(secs) => secs,
    };
    Ok(format!("{value}\n"))
}

pub(super) fn handle_incr(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
    by: i64,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    if key.is_empty() || !after.trim().is_empty() {
        return Err("usage: INCR key".to_string());
    }
    let value = cache.incr(conn, key, by).map_err(|e| e.to_string())?;
    Ok(format!("{value}\n"))
}

pub(super) fn handle_incrby(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let Some((key, by)) = split_two(rest) else {
        return Err("usage: INCRBY key increment".to_string());
    };
    let by: i64 = by
        .parse()
        .map_err(|_| "value is not an integer".to_string())?;
    let value = cache.incr(conn, key, by).map_err(|e| e.to_string())?;
    Ok(format!("{value}\n"))
}

pub(super) fn handle_append(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    let value = after.trim_start_matches(' ');
    if key.is_empty() || value.is_empty() {
        return Err("usage: APPEND key value".to_string());
    }
    let full = cache.append(conn, key, value).map_err(|e| e.to_string())?;
    Ok(format!("{}\n", full.len()))
}
