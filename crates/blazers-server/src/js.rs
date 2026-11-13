//! Embedded JavaScript runtime for edge functions, backed by QuickJS
//! (via `rquickjs`). Each invocation runs in a fresh runtime + context —
//! no shared state, no I/O, no timers. Synchronous only.

use rquickjs::{Context, Runtime};
use serde_json::Value;

use blazers_core::models::AppError;

/// 64 MiB heap cap per invocation.
const MAX_MEMORY: usize = 64 * 1024 * 1024;
/// 8 MiB native stack cap per invocation.
const MAX_STACK: usize = 8 * 1024 * 1024;

fn js_err(e: rquickjs::Error) -> AppError {
    AppError::BadRequest(format!("function error: {e}"))
}

/// Runs `code` in a sandboxed QuickJS context and returns the JSON value
/// produced by `handler(input)` (or `main(input)` as a fallback).
///
/// The function receives one argument — `input`, whatever the caller passed —
/// and must return a JSON-serializable value. There is no `fetch`, no
/// filesystem, no timers and no host bindings; pure computation only.
pub fn run(code: &str, input: &Value) -> Result<Value, AppError> {
    let rt = Runtime::new().map_err(|e| AppError::Internal(format!("js runtime: {e}")))?;
    rt.set_memory_limit(MAX_MEMORY);
    rt.set_max_stack_size(MAX_STACK);
    let ctx = Context::full(&rt).map_err(|e| AppError::Internal(format!("js context: {e}")))?;

    let source = format!(
        "{code}\n;(() => {{\n\
         \x20 const f = typeof handler === 'function' ? handler\n\
         \x20   : (typeof main === 'function' ? main : null);\n\
         \x20 if (!f) return JSON.stringify({{ __error: 'no handler or main function defined' }});\n\
         \x20 try {{\n\
         \x20   return JSON.stringify(f(JSON.parse(__blazers_input)) ?? null);\n\
         \x20 }} catch (e) {{\n\
         \x20   return JSON.stringify({{ __error: String(e && e.message ? e.message : e) }});\n\
         \x20 }}\n\
         }})()"
    );

    let out: String = ctx
        .with(|ctx| -> rquickjs::Result<String> {
            ctx.globals().set("__blazers_input", input.to_string())?;
            ctx.eval(source.as_bytes())
        })
        .map_err(js_err)?;

    let value: Value =
        serde_json::from_str(&out).map_err(|e| AppError::Internal(format!("js result: {e}")))?;
    if let Some(err) = value.get("__error").and_then(|v| v.as_str()) {
        return Err(AppError::BadRequest(format!("function error: {err}")));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn handler_receives_input() {
        let out = run(
            "function handler(input) { return { doubled: input.n * 2 }; }",
            &json!({"n": 21}),
        )
        .unwrap();
        assert_eq!(out, json!({"doubled": 42}));
    }

    #[test]
    fn main_fallback() {
        let out = run("function main() { return 'ok'; }", &json!(null)).unwrap();
        assert_eq!(out, json!("ok"));
    }

    #[test]
    fn js_exception_is_bad_request() {
        let err = run(
            "function handler() { throw new Error('boom'); }",
            &json!(null),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn missing_handler_is_error() {
        let err = run("const x = 1;", &json!(null)).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
