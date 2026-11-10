use std::io::Write;

use crate::handlers::DbHandle;
use crate::protocol::{self, Request, Tool};
use crate::tools;
use crate::transport;

mod dispatch;
use crate::{Diagnostics, SERVER_NAME, SERVER_VERSION, ToolHandler};

/// Server dispatches MCP requests to registered tools and writes frames to
/// output. The input stream is supplied per call to `serve`, so one server can
/// be driven repeatedly in tests.
pub struct Server<'a> {
    output: &'a mut dyn Write,
    diagnostics: Diagnostics<'a>,
    tools: Vec<Tool>,
    handlers: Vec<(String, ToolHandler)>,
    db: Option<DbHandle>,
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
            db: Some(db),
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
            db: Some(db),
        }
    }

    /// remote builds a server with no local database, for `--kv` proxy mode.
    pub fn remote(output: &'a mut dyn Write, diagnostics: Diagnostics<'a>) -> Server<'a> {
        Server {
            output,
            diagnostics,
            tools: Vec::new(),
            handlers: Vec::new(),
            db: None,
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
        if let Some(db) = &self.db {
            for registered in tools::register(db) {
                self.add_tool(registered.tool, registered.handler);
            }
        }
    }

    /// register_kv adds the remote-KV tool surface for `addr` (`--kv` mode).
    pub fn register_kv(&mut self, addr: &str) {
        for registered in tools::register_kv(addr) {
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
