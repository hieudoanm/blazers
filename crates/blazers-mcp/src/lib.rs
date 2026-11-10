//! Model Context Protocol server exposing Blazers data as tools.
//!
//! The server speaks newline-delimited JSON-RPC 2.0 over stdio: one frame per
//! line, diagnostics to stderr, and nothing but frames on stdout. Tools run
//! against the same SQLite database the HTTP server uses, so a model and the
//! REST API see one set of data.

pub mod handlers;
pub mod protocol;
pub mod tools;
pub mod transport;

#[cfg(test)]
mod tests;

use std::io::Write;

use handlers::DbHandle;
use protocol::{Request, Tool, ToolResult};

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

/// Server dispatches MCP requests to registered tools and writes frames to
/// output. The input stream is supplied per call to `serve`, so one server can
/// be driven repeatedly in tests.
pub struct Server<'a> {
    output: &'a mut dyn Write,
    diagnostics: Diagnostics<'a>,
    tools: Vec<Tool>,
    handlers: Vec<(String, ToolHandler)>,
    db: DbHandle,
}

impl<'a> Server<'a> {
    /// new returns a Server writing frames to output and discarding diagnostics.
    /// Tests supply a buffer so the full protocol runs without touching the
    /// process stdio. The writer is borrowed, so it must outlive the server.
    /// `run` needs stderr, so this exists for the test module alone.
    #[cfg(test)]
    pub fn new(output: &'a mut dyn Write, db: DbHandle) -> Server<'a> {
        Server {
            output,
            diagnostics: Diagnostics::Sink,
            tools: Vec::new(),
            handlers: Vec::new(),
            db,
        }
    }

    /// with_diagnostics is new with an explicit diagnostics sink.
    pub fn with_diagnostics(
        output: &'a mut dyn Write,
        diagnostics: Diagnostics<'a>,
        db: DbHandle,
    ) -> Server<'a> {
        Server {
            output,
            diagnostics,
            tools: Vec::new(),
            handlers: Vec::new(),
            db,
        }
    }

    /// add_tool registers a tool and its handler, replacing an entry with the
    /// same name.
    pub fn add_tool(&mut self, tool: Tool, handler: ToolHandler) {
        let name = tool.name.clone();
        match self.tools.iter_mut().find(|existing| existing.name == name) {
            Some(existing) => *existing = tool,
            None => self.tools.push(tool),
        }
        match self
            .handlers
            .iter_mut()
            .find(|(registered, _)| *registered == name)
        {
            Some((_, existing)) => *existing = handler,
            None => self.handlers.push((name, handler)),
        }
    }

    /// register_tools adds the Blazers tool surface, backed by the shared
    /// database rather than a stub.
    pub fn register_tools(&mut self) {
        for registered in tools::register(&self.db) {
            self.add_tool(registered.tool, registered.handler);
        }
    }

    /// serve reads and dispatches frames until EOF. A clean EOF is success.
    pub fn serve(&mut self, input: impl std::io::Read) -> std::io::Result<()> {
        let mut buffered = std::io::BufReader::new(input);
        let mut reader = transport::FrameReader::new(&mut buffered);
        loop {
            match reader.next_frame() {
                transport::Frame::Message(line) => self.handle_message(line.as_bytes()),
                transport::Frame::TooLong => {
                    self.write(protocol::error_response(
                        serde_json::Value::Null,
                        protocol::error_code::PARSE,
                        "parse error: frame too large",
                    ));
                }
                transport::Frame::Eof => return Ok(()),
                transport::Frame::Error(err) => return Err(err),
            }
        }
    }

    /// handle_message decodes one frame and dispatches it. An undecodable frame
    /// produces a parse error. A notification — a frame with no id — is never
    /// answered whatever the method, because answering one desynchronises the
    /// client. That check precedes the version check so a notification which is
    /// also malformed stays silent.
    fn handle_message(&mut self, raw: &[u8]) {
        let request: Request = match serde_json::from_slice(raw) {
            Ok(request) => request,
            Err(err) => {
                self.write(protocol::error_response(
                    serde_json::Value::Null,
                    protocol::error_code::PARSE,
                    format!("parse error: {err}"),
                ));
                return;
            }
        };

        if request.is_notification() {
            let _ = writeln!(
                self.diagnostics,
                "ignoring notification for method {}",
                request.method.as_deref().unwrap_or("<missing>")
            );
            return;
        }
        if request.jsonrpc.as_deref() != Some(protocol::JSONRPC_VERSION) {
            self.write(protocol::error_response(
                request.id.clone().unwrap_or(serde_json::Value::Null),
                protocol::error_code::INVALID_REQUEST,
                "invalid jsonrpc version",
            ));
            return;
        }

        let id = request.id.clone().unwrap_or(serde_json::Value::Null);
        match request.method.as_deref() {
            Some("initialize") => self.write(protocol::success_response(
                id,
                protocol::initialize_result(request.params.as_ref(), SERVER_NAME, SERVER_VERSION),
            )),
            Some("ping") => self.write(protocol::success_response(id, serde_json::json!({}))),
            Some("tools/list") => self.write(protocol::success_response(
                id,
                serde_json::to_value(self.list_result()).unwrap_or(serde_json::Value::Null),
            )),
            Some("tools/call") => self.handle_call(id, request.params.as_ref()),
            Some(other) => self.write(protocol::error_response(
                id,
                protocol::error_code::METHOD_NOT_FOUND,
                format!("method not found: {other}"),
            )),
            None => self.write(protocol::error_response(
                id,
                protocol::error_code::INVALID_REQUEST,
                "invalid request: method is required",
            )),
        }
    }

    /// list_result returns the registered tools sorted by name, so a client sees
    /// a stable order across runs.
    fn list_result(&self) -> protocol::ListToolsResult {
        let mut tools = self.tools.clone();
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        protocol::ListToolsResult { tools }
    }

    /// handle_call decodes the call params and runs the named handler. Absent or
    /// null params are treated as an empty object so a call with no params names
    /// no tool rather than failing to decode.
    fn handle_call(&mut self, id: serde_json::Value, params: Option<&serde_json::Value>) {
        let call: protocol::ToolCallParams = match protocol::object_or_empty(params) {
            Ok(call) => call,
            Err(message) => {
                self.write(protocol::error_response(
                    id,
                    protocol::error_code::INVALID_PARAMS,
                    format!("invalid params: {message}"),
                ));
                return;
            }
        };

        let handler = self
            .handlers
            .iter()
            .find(|(name, _)| *name == call.name)
            .map(|(_, handler)| handler);

        match handler {
            None => self.write(protocol::error_response(
                id,
                protocol::error_code::METHOD_NOT_FOUND,
                format!("tool not found: {}", call.name),
            )),
            Some(handler) => {
                let result = handler(&call.arguments_json());
                match serde_json::to_value(result) {
                    Ok(value) => self.write(protocol::success_response(id, value)),
                    Err(err) => self.write(protocol::error_response(
                        id,
                        protocol::error_code::INTERNAL,
                        format!("could not encode tool result: {err}"),
                    )),
                }
            }
        }
    }

    /// write emits one compact frame. Responses are marshalled compactly so a
    /// reply never spans lines, which would desynchronise a line-oriented client.
    fn write(&mut self, response: protocol::Response) {
        let encoded = match serde_json::to_string(&response) {
            Ok(encoded) => encoded,
            Err(err) => {
                let _ = writeln!(self.diagnostics, "could not encode response: {err}");
                return;
            }
        };
        if let Err(err) = writeln!(self.output, "{encoded}") {
            let _ = writeln!(self.diagnostics, "could not write response: {err}");
        }
    }
}
