use super::*;

impl<'a> Server<'a> {
    /// handle_message decodes one frame and dispatches it. An undecodable frame
    /// produces a parse error. A notification — a frame with no id — is never
    /// answered whatever the method, because answering one desynchronises the
    /// client. That check precedes the version check so a notification which is
    /// also malformed stays silent.
    pub(super) fn handle_message(&mut self, raw: &[u8]) {
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
}
