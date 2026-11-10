//! Model Context Protocol server exposing Blazers data as tools.
//!
//! The server speaks newline-delimited JSON-RPC 2.0 over stdio: one frame per
//! line, diagnostics to stderr, and nothing but frames on stdout. Tools run
//! against the same SQLite database the HTTP server uses, so a model and the
//! REST API see one set of data.

pub mod handlers;
pub mod kv;
pub mod protocol;
pub mod tools;
pub mod transport;

use std::io::Write;

use protocol::ToolResult;

/// The server name reported to a client during initialize.
pub const SERVER_NAME: &str = "blazers-mcp";

/// The version reported to a client during initialize.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ToolHandler executes one tool call. It receives the raw arguments and returns
/// a tool result rather than an error, so a model can see and react to a failure.
pub type ToolHandler = Box<dyn Fn(&str) -> ToolResult>;

/// Runs the MCP server until stdin reaches EOF, which is how a client signals
/// shutdown. `diagnostics` receives notification notices and write failures;
/// stdout stays reserved for JSON-RPC frames.
pub fn run(diagnostics: &mut dyn Write) -> std::io::Result<()> {
    let db = handlers::open_db()?;
    let mut stdout = std::io::stdout();
    let mut server = Server::with_diagnostics(&mut stdout, Diagnostics::Stream(diagnostics), db);
    server.register_tools();
    server.serve(std::io::stdin())
}

/// Runs the MCP server against a remote `blazers kv` listener instead of the
/// local database — kevin's `mcp serve --addr` equivalent. `addr` is
/// `host:port`; `BLAZERS_KV_PASSWORD` authenticates when the remote requires it.
pub fn run_remote(addr: &str, diagnostics: &mut dyn Write) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    let mut server = Server::remote(&mut stdout, Diagnostics::Stream(diagnostics));
    server.register_kv(addr);
    server.serve(std::io::stdin())
}

/// Diagnostics receives notification notices and write failures. Owning the
/// sink rather than borrowing it keeps Server free of a lifetime parameter, so
/// tests can construct one inline.
pub enum Diagnostics<'a> {
    /// Discarded, which is what tests that do not assert on diagnostics want.
    /// This crate is a binary, so nothing outside the test module needs it.
    #[cfg(test)]
    Sink,
    /// Written to the caller's stream; stdout is never valid here.
    Stream(&'a mut dyn Write),
}

impl Write for Diagnostics<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(test)]
            Diagnostics::Sink => Ok(buf.len()),
            Diagnostics::Stream(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(test)]
            Diagnostics::Sink => Ok(()),
            Diagnostics::Stream(stream) => stream.flush(),
        }
    }
}

mod server;

pub use server::Server;

#[cfg(test)]
mod tests;
