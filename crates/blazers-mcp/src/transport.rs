//! A bounded, newline-delimited frame reader.
//!
//! The transport is one JSON document per line, so this reads bytes up to a
//! newline rather than calling `read_until`, which would buffer an unbounded
//! line before any cap could apply.

use std::io::{self, BufRead};

/// Caps a single JSON-RPC frame. A larger frame is reported rather than
/// buffered, so a client cannot grow the heap without bound.
pub const MAX_FRAME_BYTES: usize = 8 << 20;

/// One frame read from the request stream.
#[derive(Debug)]
pub enum Frame {
    /// A complete frame, including its newline when one was present.
    Message(String),
    /// A frame beyond [MAX_FRAME_BYTES]; its tail has been discarded.
    TooLong,
    /// The stream ended.
    Eof,
    /// The underlying reader failed.
    Error(io::Error),
}

impl PartialEq for Frame {
    /// Compares by variant and payload, treating two read errors as equal so a
    /// test can assert on the kind of failure rather than its identity.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Frame::Message(a), Frame::Message(b)) => a == b,
            (Frame::TooLong, Frame::TooLong) | (Frame::Eof, Frame::Eof) => true,
            (Frame::Error(a), Frame::Error(b)) => a.kind() == b.kind(),
            _ => false,
        }
    }
}

/// Reads frames from a buffered reader.
pub struct FrameReader<'a, R: BufRead> {
    reader: &'a mut R,
}

impl<'a, R: BufRead> FrameReader<'a, R> {
    /// Wraps a reader.
    pub fn new(reader: &'a mut R) -> Self {
        FrameReader { reader }
    }

    /// Reads the next frame, or [Frame::Eof] at end of stream.
    pub fn next_frame(&mut self) -> Frame {
        let mut frame: Vec<u8> = Vec::new();
        let mut overflowed = false;

        loop {
            let available = match self.reader.fill_buf() {
                Ok(buffer) => buffer,
                Err(err) => return Frame::Error(err),
            };
            if available.is_empty() {
                // End of stream. Bytes already read form a final frame: a client
                // whose last frame lacks a trailing newline is normal, so this
                // must not be reported as a parse error.
                return finish(frame, overflowed);
            }

            match available.iter().position(|byte| *byte == b'\n') {
                Some(index) => {
                    append(&mut frame, &available[..=index], &mut overflowed);
                    self.reader.consume(index + 1);
                    if overflowed {
                        return Frame::TooLong;
                    }
                    return match String::from_utf8(frame) {
                        Ok(text) => Frame::Message(text),
                        // Invalid UTF-8 cannot be JSON, so report an empty frame;
                        // the server answers with a parse error.
                        Err(_) => Frame::Message(String::new()),
                    };
                }
                None => {
                    let len = available.len();
                    append(&mut frame, available, &mut overflowed);
                    self.reader.consume(len);
                }
            }
        }
    }
}

/// Turns accumulated state into a frame at end of stream.
fn finish(frame: Vec<u8>, overflowed: bool) -> Frame {
    if overflowed {
        return Frame::TooLong;
    }
    match String::from_utf8(frame) {
        Ok(text) if text.is_empty() => Frame::Eof,
        Ok(text) => Frame::Message(text),
        Err(_) => Frame::Message(String::new()),
    }
}

/// Copies a chunk into the frame, stopping once the cap is exceeded. The bytes
/// are still consumed from the reader so the tail is discarded rather than
/// parsed as a new frame.
fn append(frame: &mut Vec<u8>, chunk: &[u8], overflowed: &mut bool) {
    if *overflowed {
        return;
    }
    if frame.len() + chunk.len() > MAX_FRAME_BYTES {
        *overflowed = true;
        frame.clear();
        frame.shrink_to_fit();
        return;
    }
    frame.extend_from_slice(chunk);
}
