//! Blazers — a single-binary Back-end as a Service.
//!
//! `blazers` starts the HTTP server; `blazers mcp serve` starts the MCP
//! server on stdio. Both share the same SQLite database.

use std::io::Write;

/// USAGE lists the accepted command lines. The default is the HTTP server, so
/// `blazers` and `blazers serve` both start it.
const USAGE: &str = "\
Blazers - blazing fast Back-end as a Service

Usage:
  blazers              Start the HTTP server
  blazers serve        Start the HTTP server
  blazers kv           Start the KV (Redis-style) TCP server
  blazers kv --port N  KV server on port N (default 6379)
  blazers mcp serve    Start the MCP server on stdio
  blazers mcp serve --kv HOST:PORT
                       MCP server proxying a remote blazers kv server
  blazers --help       Show this message
  blazers --version    Show the version";

/// Reports whether the arguments select the stdio MCP server, with the
/// optional `--kv HOST:PORT` remote-store flag.
fn mcp_args(args: &[String]) -> Option<Option<String>> {
    if args.first().map(String::as_str) != Some("mcp")
        || args.get(1).map(String::as_str) != Some("serve")
    {
        return None;
    }
    let mut kv = None;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--kv" {
            kv = Some(args.get(i + 1)?.clone());
            i += 2;
        } else {
            return None;
        }
    }
    Some(kv)
}

/// Parses `blazers kv [--port N] [--bind ADDR]` into (bind, port).
fn kv_args(args: &[String]) -> Option<(String, u16)> {
    if args.first().map(String::as_str) != Some("kv") {
        return None;
    }
    let mut bind = "127.0.0.1".to_string();
    let mut port = 6379u16;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                port = args.get(i + 1)?.parse().ok()?;
                i += 2;
            }
            "--bind" => {
                bind = args.get(i + 1)?.clone();
                i += 2;
            }
            _ => return None,
        }
    }
    Some((bind, port))
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        return;
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!(
            "blazers {} (build {})",
            env!("CARGO_PKG_VERSION"),
            env!("BLAZERS_BUILD_TIME")
        );
        return;
    }
    if let Some(kv_addr) = mcp_args(&args) {
        // The MCP server is synchronous and reads stdin to EOF, so it runs on
        // this thread. It opens the same SQLite database as the HTTP server,
        // or proxies a remote `blazers kv` listener when --kv is given.
        let mut stderr = std::io::stderr();
        let result = match kv_addr {
            Some(addr) => blazers_mcp::run_remote(&addr, &mut stderr),
            None => blazers_mcp::run(&mut stderr),
        };
        if let Err(err) = result {
            let _ = writeln!(stderr, "blazers mcp serve: {err}");
            std::process::exit(1);
        }
        return;
    }
    if let Some((bind, port)) = kv_args(&args) {
        if let Err(err) = blazers_server::kvproto::serve_kv(&bind, port).await {
            eprintln!("blazers kv: {err}");
            std::process::exit(1);
        }
        return;
    }
    // Bare `serve` and no arguments both start the HTTP server; anything else
    // is a typo worth reporting rather than silently ignoring.
    let starts_http = args.is_empty() || (args.len() == 1 && args[0] == "serve");
    if !starts_http {
        eprintln!("{USAGE}");
        std::process::exit(2);
    }

    blazers_server::app::serve_http().await
}
