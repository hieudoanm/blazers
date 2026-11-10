use super::*;

pub(super) fn handle_hset(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let tokens = split_all(rest);
    if tokens.len() < 3 || tokens.len().is_multiple_of(2) {
        return Err("usage: HSET key field value [field value ...]".to_string());
    }
    let mut fields = serde_json::Map::new();
    let (pairs, _) = tokens[1..].as_chunks::<2>();
    for pair in pairs {
        fields.insert(pair[0].clone(), Value::String(pair[1].clone()));
    }
    let n = cache
        .hash_set(conn, &tokens[0], fields)
        .map_err(|e| e.to_string())?;
    Ok(format!("{n}\n"))
}

pub(super) fn handle_hget(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let tokens = split_all(rest);
    if tokens.len() != 2 {
        return Err("usage: HGET key field".to_string());
    }
    match cache
        .hash_get(conn, &tokens[0], &tokens[1])
        .map_err(|e| e.to_string())?
    {
        Some(v) => Ok(v.as_str().unwrap_or("").to_string() + "\n"),
        None => Ok("(nil)\n".to_string()),
    }
}

pub(super) fn handle_hgetall(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    if key.is_empty() || !after.trim().is_empty() {
        return Err("usage: HGETALL key".to_string());
    }
    let fields = cache.hash_get_all(conn, key).map_err(|e| e.to_string())?;
    let mut parts = Vec::new();
    for (f, v) in fields {
        parts.push(f);
        parts.push(v.as_str().unwrap_or("").to_string());
    }
    Ok(if parts.is_empty() {
        "\n".to_string()
    } else {
        parts.join(" ") + "\n"
    })
}

pub(super) fn handle_hdel(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let tokens = split_all(rest);
    if tokens.len() < 2 {
        return Err("usage: HDEL key field [field ...]".to_string());
    }
    let n = cache
        .hash_del(conn, &tokens[0], &tokens[1..])
        .map_err(|e| e.to_string())?;
    Ok(format!("{n}\n"))
}

pub(super) fn handle_hlen(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let (key, after) = split_token(rest);
    if key.is_empty() || !after.trim().is_empty() {
        return Err("usage: HLEN key".to_string());
    }
    let n = cache.hash_len(conn, key).map_err(|e| e.to_string())?;
    Ok(format!("{n}\n"))
}
