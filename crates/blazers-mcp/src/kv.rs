//! Remote KV store client: speaks the `blazers kv` inline protocol over TCP,
//! mirroring kevin's `TcpStore` MCP proxy mode.
//!
//! Every call opens a short-lived connection, sends one command line and
//! reads one reply line. When `BLAZERS_KV_PASSWORD` is set the remote server
//! requires AUTH, so each connection authenticates first.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Sends one command line to `addr` (`host:port`) and returns the reply line
/// without its trailing newline.
pub fn call(addr: &str, command: &str) -> std::io::Result<String> {
    let stream = TcpStream::connect_timeout(
        &addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| std::io::Error::other("unresolvable address"))?,
        Duration::from_secs(5),
    )?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    if let Ok(password) = std::env::var("BLAZERS_KV_PASSWORD") {
        writeln!(writer, "AUTH {password}")?;
        let mut reply = String::new();
        reader.read_line(&mut reply)?;
        if !reply.starts_with("+OK") && reply.trim() != "OK" {
            return Err(std::io::Error::other(format!(
                "remote AUTH failed: {}",
                reply.trim()
            )));
        }
    }

    writeln!(writer, "{command}")?;
    let mut reply = String::new();
    if reader.read_line(&mut reply)? == 0 {
        return Err(std::io::Error::other("connection closed without reply"));
    }
    Ok(reply.trim_end_matches(['\r', '\n']).to_string())
}
