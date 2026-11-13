use super::basic::*;
use super::hashes::*;
use super::lists::*;
use super::ops::*;
use super::*;

pub(super) fn handle_auth(rest: &str, authed: &mut bool, require_auth: bool) -> (String, bool) {
    let (password, after) = split_token(rest);
    if !after.trim().is_empty() || password.is_empty() {
        return ("ERR usage: AUTH password\n".to_string(), true);
    }
    if !require_auth {
        return (
            "ERR Client sent AUTH, but no password is set\n".to_string(),
            true,
        );
    }
    match std::env::var("BLAZERS_KV_PASSWORD") {
        Ok(expected) if expected == password => {
            *authed = true;
            ("OK\n".to_string(), true)
        }
        _ => (
            "WRONGPASS invalid username-password pair\n".to_string(),
            true,
        ),
    }
}

pub(super) fn dispatch(
    cmd: &str,
    rest: &str,
    cache: &CacheStore,
    conn: &rusqlite::Connection,
) -> Result<String, String> {
    match cmd {
        "PING" => {
            if rest.trim().is_empty() {
                Ok("PONG\n".to_string())
            } else {
                Err("usage: PING".to_string())
            }
        }
        "SET" => handle_set(rest, cache, conn),
        "GET" => handle_get(rest, cache, conn),
        "DEL" => handle_del(rest, cache, conn),
        "EXISTS" => handle_exists(rest, cache, conn),
        "KEYS" => handle_keys(rest, cache),
        "TYPE" => handle_type(rest, cache, conn),
        "LEN" => handle_len(rest, cache),
        "FLUSHALL" | "FLUSHDB" => handle_flush(rest, cache, conn),
        "EXPIRE" => handle_expire(rest, cache, conn),
        "TTL" => handle_ttl(rest, cache, conn),
        "INCR" => handle_incr(rest, cache, conn, 1),
        "DECR" => handle_incr(rest, cache, conn, -1),
        "INCRBY" => handle_incrby(rest, cache, conn),
        "APPEND" => handle_append(rest, cache, conn),
        "LPUSH" | "RPUSH" => handle_list_push(rest, cache, conn, cmd == "LPUSH"),
        "LPOP" | "RPOP" => handle_list_pop(rest, cache, conn, cmd == "LPOP"),
        "LRANGE" => handle_lrange(rest, cache, conn),
        "HSET" => handle_hset(rest, cache, conn),
        "HGET" => handle_hget(rest, cache, conn),
        "HGETALL" => handle_hgetall(rest, cache, conn),
        "HDEL" => handle_hdel(rest, cache, conn),
        "HLEN" => handle_hlen(rest, cache, conn),
        _ => Err(format!("unknown command '{cmd}'")),
    }
}
