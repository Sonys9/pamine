//! Parser tests: framing, fragmentation and buffer bookkeeping.
//!
//! All packet enums are currently empty stubs, so a well-formed frame still
//! fails with `UnknownOpcode`. These tests therefore assert on *framing*
//! behaviour — how many bytes were consumed, whether a partial frame is
//! reported as `Ok(None)` — rather than on decoded packet contents.

use pcore::encoder::varint;

use super::{Parser, Side, State};

/// Builds an uncompressed frame: `len:VarInt body`.
fn frame(body: &[u8]) -> Vec<u8> {
    let mut out = varint(body.len() as i32);
    out.extend_from_slice(body);
    out
}

/// A body with a single opcode byte. No packet enum has this opcode, so parsing
/// reaches the dispatcher and fails with `UnknownOpcode` — which is exactly what
/// we want: it proves the frame was split correctly.
fn unknown_body() -> Vec<u8> {
    vec![0x7f]
}

#[test]
fn empty_buffer_returns_none() {
    let mut parser = Parser::new();
    assert!(parser.try_parse(Side::C2S).unwrap().is_none());
}

#[test]
fn incomplete_packet_returns_none() {
    let mut parser = Parser::new();
    let full = frame(&unknown_body());
    parser.update(Side::C2S, &full[..full.len() - 1]);

    assert!(parser.try_parse(Side::C2S).unwrap().is_none());
    // Nothing was consumed, so the bytes are still there.
    assert!(!parser.is_empty(Side::C2S));
}

#[test]
fn fragmented_packet_completes_after_second_read() {
    let mut parser = Parser::new();
    let full = frame(&unknown_body());
    let (a, b) = full.split_at(1);

    parser.update(Side::C2S, a);
    assert!(parser.try_parse(Side::C2S).unwrap().is_none());

    parser.update(Side::C2S, b);
    // The frame is now complete, so parsing gets past framing and fails on the
    // unknown opcode instead of reporting "need more data".
    assert!(parser.try_parse(Side::C2S).is_err());
}

#[test]
fn negative_length_is_an_error() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &varint(-1));
    assert!(parser.try_parse(Side::C2S).is_err());
}

#[test]
fn frame_is_consumed_exactly_once() {
    let mut parser = Parser::new();
    // Two frames back to back.
    let mut bytes = frame(&unknown_body());
    bytes.extend_from_slice(&frame(&unknown_body()));
    parser.update(Side::C2S, &bytes);

    // First frame: framing succeeds, dispatch fails on the unknown opcode.
    assert!(parser.try_parse(Side::C2S).is_err());
    // The cursor advanced by exactly one frame, so the second one is still there.
    assert!(!parser.is_empty(Side::C2S));

    parser.clear(Side::C2S);
    assert!(parser.try_parse(Side::C2S).is_err());
    assert!(parser.is_empty(Side::C2S));
}

#[test]
fn clear_compacts_the_buffer() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &frame(&unknown_body()));
    let _ = parser.try_parse(Side::C2S);

    parser.clear(Side::C2S);
    assert!(parser.is_empty(Side::C2S));
    assert!(parser.try_parse(Side::C2S).unwrap().is_none());
}

#[test]
fn reset_restores_initial_state() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &frame(&unknown_body()));
    let _ = parser.try_parse(Side::C2S);

    parser.reset();
    assert_eq!(parser.state(), State::Handshake);
    assert!(parser.is_empty(Side::C2S));
    assert!(parser.try_parse(Side::C2S).unwrap().is_none());
}

#[test]
fn sides_are_independent() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &frame(&unknown_body()));

    // Bytes on C2S must not be visible on S2C.
    assert!(parser.try_parse(Side::S2C).unwrap().is_none());
    assert!(!parser.is_empty(Side::C2S));
}
