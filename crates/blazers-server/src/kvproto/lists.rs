use super::*;

pub(super) fn handle_list_push(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
    left: bool,
) -> Result<String, String> {
    let tokens = split_all(rest);
    if tokens.len() < 2 {
        return Err("usage: LPUSH/RPUSH key value [value ...]".to_string());
    }
    let values: Vec<Value> = tokens[1..]
        .iter()
        .map(|v| Value::String(v.clone()))
        .collect();
    let len = cache
        .list_push(conn, &tokens[0], values, left)
        .map_err(|e| e.to_string())?;
    Ok(format!("{len}\n"))
}

pub(super) fn handle_list_pop(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
    left: bool,
) -> Result<String, String> {
    let tokens = split_all(rest);
    if tokens.is_empty() || tokens.len() > 2 {
        return Err("usage: LPOP/RPOP key [count]".to_string());
    }
    let count = match tokens.get(1) {
        None => None,
        Some(raw) => Some(
            raw.parse::<usize>()
                .map_err(|_| "value is not an integer".to_string())?,
        ),
    };
    match cache
        .list_pop(conn, &tokens[0], left, count)
        .map_err(|e| e.to_string())?
    {
        None => Ok("(nil)\n".to_string()),
        Some(Value::Array(items)) => Ok(items
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect::<Vec<_>>()
            .join(" ")
            + "\n"),
        Some(v) => Ok(v.as_str().unwrap_or("").to_string() + "\n"),
    }
}

pub(super) fn handle_lrange(
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    let tokens = split_all(rest);
    if tokens.len() != 3 {
        return Err("usage: LRANGE key start stop".to_string());
    }
    let start: i64 = tokens[1]
        .parse()
        .map_err(|_| "value is not an integer".to_string())?;
    let stop: i64 = tokens[2]
        .parse()
        .map_err(|_| "value is not an integer".to_string())?;
    let items = cache
        .list_range(conn, &tokens[0], start, stop)
        .map_err(|e| e.to_string())?;
    Ok(items
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect::<Vec<_>>()
        .join(" ")
        + "\n")
}
