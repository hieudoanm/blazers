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
  blazers mcp serve    Start the MCP server on stdio
  blazers --help       Show this message
  blazers --version    Show the version";

/// Reports whether the arguments select the stdio MCP server.
///
/// `mcp serve` is checked as a pair so an argument list of `mcp` alone falls
/// through to the help text rather than starting a server that ignores it.
fn is_mcp_serve(args: &[String]) -> bool {
    args.len() == 2 && args[0] == "mcp" && args[1] == "serve"
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
    if is_mcp_serve(&args) {
        // The MCP server is synchronous and reads stdin to EOF, so it runs on
        // this thread. It opens the same SQLite database as the HTTP server.
        let mut stderr = std::io::stderr();
        if let Err(err) = blazers_mcp::run(&mut stderr) {
            let _ = writeln!(stderr, "blazers mcp serve: {err}");
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
