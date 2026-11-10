use super::*;

// --- Remote KV tools (`blazers mcp serve --kv`) ---------------------------
//
// These proxy the `blazers kv` inline protocol, mirroring kevin's TcpStore
// mode: the MCP server holds no database of its own.

/// Sends one command to the remote KV server and maps the reply: `ERR`,
/// `NOAUTH` and `WRONGPASS` replies become tool errors so a model can react.
fn kv_reply(addr: &str, command: &str) -> ToolResult {
    match crate::kv::call(addr, command) {
        Ok(reply)
            if reply.starts_with("ERR")
                || reply.starts_with("NOAUTH")
                || reply.starts_with("WRONGPASS") =>
        {
            error_result(reply)
        }
        Ok(reply) => text_result(reply),
        Err(err) => error_result(format!("kv {addr}: {err}")),
    }
}

/// Arguments of `blazers_kv_set`.
#[derive(Debug, serde::Deserialize)]
pub struct KvSetArgs {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub ex: Option<i64>,
}

/// Arguments of the single-key KV tools.
#[derive(Debug, serde::Deserialize)]
pub struct KvKeyArgs {
    pub key: String,
}

/// Arguments of `blazers_kv_del`.
#[derive(Debug, serde::Deserialize)]
pub struct KvDelArgs {
    #[serde(default)]
    pub keys: Vec<String>,
}

/// Arguments of `blazers_kv_keys`.
#[derive(Debug, serde::Deserialize)]
pub struct KvKeysArgs {
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Arguments of `blazers_kv_expire`.
#[derive(Debug, serde::Deserialize)]
pub struct KvExpireArgs {
    pub key: String,
    pub seconds: i64,
}

macro_rules! kv_tool {
    ($name:ident, $args:ty, $body:expr) => {
        pub fn $name(addr: &str, arguments: &str) -> ToolResult {
            let raw = if arguments.trim().is_empty() {
                "{}"
            } else {
                arguments
            };
            let args: $args = match parse_args(raw) {
                Ok(args) => args,
                Err(message) => return error_result(message),
            };
            kv_reply(addr, &($body)(&args))
        }
    };
}

kv_tool!(
    handle_kv_ping,
    serde_json::Value,
    |_: &serde_json::Value| { "PING".to_string() }
);

pub fn handle_kv_set(addr: &str, arguments: &str) -> ToolResult {
    let args: KvSetArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    let command = match args.ex {
        Some(seconds) if seconds > 0 => format!("SET {} {} EX {seconds}", args.key, args.value),
        _ => format!("SET {} {}", args.key, args.value),
    };
    kv_reply(addr, &command)
}

kv_tool!(handle_kv_get, KvKeyArgs, |a: &KvKeyArgs| {
    format!("GET {}", a.key)
});

pub fn handle_kv_del(addr: &str, arguments: &str) -> ToolResult {
    let args: KvDelArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    if args.keys.is_empty() {
        return error_result("blazers_kv_del: keys is required");
    }
    kv_reply(addr, &format!("DEL {}", args.keys.join(" ")))
}

pub fn handle_kv_exists(addr: &str, arguments: &str) -> ToolResult {
    let args: KvKeyArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    kv_reply(addr, &format!("EXISTS {}", args.key))
}

pub fn handle_kv_keys(addr: &str, arguments: &str) -> ToolResult {
    let args: KvKeysArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    let command = match args.pattern {
        Some(pattern) if !pattern.is_empty() => format!("KEYS {pattern}"),
        _ => "KEYS".to_string(),
    };
    kv_reply(addr, &command)
}

kv_tool!(handle_kv_len, serde_json::Value, |_: &serde_json::Value| {
    "LEN".to_string()
});

kv_tool!(handle_kv_ttl, KvKeyArgs, |a: &KvKeyArgs| {
    format!("TTL {}", a.key)
});

pub fn handle_kv_expire(addr: &str, arguments: &str) -> ToolResult {
    let args: KvExpireArgs = match parse_args(arguments) {
        Ok(args) => args,
        Err(message) => return error_result(message),
    };
    kv_reply(addr, &format!("EXPIRE {} {}", args.key, args.seconds))
}
