use super::*;

pub(super) fn handle_set(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    let value = after.trim_start_matches(' ');
    if key.is_empty() || value.is_empty() {
        return Err("usage: SET key value [EX seconds]".to_string());
    }
    if let Some(idx) = value.to_ascii_uppercase().rfind(" EX ") {
        let seconds = value[idx + 4..].trim();
        let ttl: i64 = seconds
            .parse()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| "invalid expire time".to_string())?;
        cache
            .set(conn, key, &value[..idx], ttl)
            .map_err(|e| e.to_string())?;
        return Ok("OK\n".to_string());
    }
    cache.set(conn, key, value, 0).map_err(|e| e.to_string())?;
    Ok("OK\n".to_string())
}

pub(super) fn handle_get(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    if key.is_empty() || !after.trim().is_empty() {
        return Err("usage: GET key".to_string());
    }
    match cache.get(conn, key).map_err(|e| e.to_string())? {
        Some(entry) => Ok(entry.value + "\n"),
        None => Ok("(nil)\n".to_string()),
    }
}

pub(super) fn handle_del(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let keys = split_all(rest);
    if keys.is_empty() {
        return Err("usage: DEL key [key ...]".to_string());
    }
    let n = cache.delete_many(conn, &keys).map_err(|e| e.to_string())?;
    Ok(format!("{n}\n"))
}

pub(super) fn handle_exists(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    if key.is_empty() || !after.trim().is_empty() {
        return Err("usage: EXISTS key".to_string());
    }
    let n = if cache.exists(conn, key).map_err(|e| e.to_string())? {
        "1"
    } else {
        "0"
    };
    Ok(format!("{n}\n"))
}

pub(super) fn handle_keys(rest: &str, cache: &CacheStore) -> Result<String, String> {
    let (pattern, after) = split_token(rest);
    if !after.trim().is_empty() {
        return Err("usage: KEYS [pattern]".to_string());
    }
    let pattern = if pattern.is_empty() { "*" } else { pattern };
    let mut keys: Vec<String> = cache
        .keys()
        .into_iter()
        .filter(|k| glob_match(pattern, k))
        .collect();
    keys.sort();
    Ok(if keys.is_empty() {
        "\n".to_string()
    } else {
        keys.join(" ") + "\n"
    })
}

/// Minimal glob: `*` matches any run, `?` matches one char.
pub(super) fn glob_match(pattern: &str, text: &str) -> bool {
    fn inner(p: &[u8], t: &[u8]) -> bool {
        match p.first() {
            None => t.is_empty(),
            Some(b'*') => (0..=t.len()).any(|i| inner(&p[1..], &t[i..])),
            Some(b'?') => !t.is_empty() && inner(&p[1..], &t[1..]),
            Some(&c) => !t.is_empty() && t[0] == c && inner(&p[1..], &t[1..]),
        }
    }
    inner(pattern.as_bytes(), text.as_bytes())
}

pub(super) fn handle_type(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    if key.is_empty() || !after.trim().is_empty() {
        return Err("usage: TYPE key".to_string());
    }
    Ok(cache
        .kind(conn, key)
        .map_err(|e| e.to_string())?
        .to_string()
        + "\n")
}

pub(super) fn handle_len(rest: &str, cache: &CacheStore) -> Result<String, String> {
    if !rest.trim().is_empty() {
        return Err("usage: LEN".to_string());
    }
    Ok(format!("{}\n", cache.keys().len()))
}

pub(super) fn handle_flush(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    if !rest.trim().is_empty() {
        return Err("usage: FLUSHALL".to_string());
    }
    cache.flush(conn).map_err(|e| e.to_string())?;
    Ok("OK\n".to_string())
}
