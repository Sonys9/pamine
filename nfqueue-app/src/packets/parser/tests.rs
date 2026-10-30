//! Parser tests: framing, fragmentation and state transitions.

use pcore::encoder::varint;

use super::{Parser, Side, State};

/// Builds an uncompressed frame: `len:VarInt body`.
fn frame(body: &[u8]) -> Vec<u8> {
    let mut out = varint(body.len() as i32);
    out.extend_from_slice(body);
    out
}

/// A C2S handshake packet body (opcode + fields).
fn handshake_body(next_state: i32) -> Vec<u8> {
    let mut body = vec![0x00]; // opcode
    body.extend_from_slice(&varint(760)); // protocol version
    body.push(9); // address length
    body.extend_from_slice(b"localhost");
    body.extend_from_slice(&25565u16.to_be_bytes());
    body.extend_from_slice(&varint(next_state));
    body
}

#[test]
fn parses_single_packet() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &frame(&handshake_body(2)));

    let packet = parser.try_parse(Side::C2S).unwrap();
    assert!(packet.is_some());
    assert_eq!(parser.state(), State::Login);
    assert!(parser.is_empty(Side::C2S));
}

#[test]
fn parses_two_packets_from_one_read() {
    let mut parser = Parser::new();
    // Handshake with next_state=1 moves us to Status, where the state does
    // not change again, so both following packets parse the same way.
    parser.update(Side::C2S, &frame(&handshake_body(1)));
    assert!(parser.try_parse(Side::C2S).unwrap().is_some());
    parser.clear(Side::C2S);
    assert_eq!(parser.state(), State::Status);

    // Two Status Request packets (opcode 0x00, no fields) in one read.
    let mut bytes = frame(&[0x00]);
    bytes.extend_from_slice(&frame(&[0x00]));
    parser.update(Side::C2S, &bytes);

    assert!(parser.try_parse(Side::C2S).unwrap().is_some());
    parser.clear(Side::C2S);
    assert!(parser.try_parse(Side::C2S).unwrap().is_some());
    parser.clear(Side::C2S);
    assert!(parser.is_empty(Side::C2S));
}

#[test]
fn incomplete_packet_returns_none() {
    let mut parser = Parser::new();
    let full = frame(&handshake_body(2));
    parser.update(Side::C2S, &full[..full.len() - 1]);

    assert!(parser.try_parse(Side::C2S).unwrap().is_none());
    // Nothing was consumed, so the bytes are still there.
    assert!(!parser.is_empty(Side::C2S));
}

#[test]
fn empty_buffer_returns_none() {
    let mut parser = Parser::new();
    assert!(parser.try_parse(Side::C2S).unwrap().is_none());
}

#[test]
fn fragmented_packet_completes_after_second_read() {
    let mut parser = Parser::new();
    let full = frame(&handshake_body(2));
    let (a, b) = full.split_at(5);

    parser.update(Side::C2S, a);
    assert!(parser.try_parse(Side::C2S).unwrap().is_none());

    parser.update(Side::C2S, b);
    assert!(parser.try_parse(Side::C2S).unwrap().is_some());
}

#[test]
fn negative_length_is_an_error() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &varint(-1));
    assert!(parser.try_parse(Side::C2S).is_err());
}

#[test]
fn reset_restores_initial_state() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &frame(&handshake_body(2)));
    parser.try_parse(Side::C2S).unwrap();
    assert_eq!(parser.state(), State::Login);

    parser.reset();
    assert_eq!(parser.state(), State::Handshake);
    assert!(parser.is_empty(Side::C2S));
}

#[test]
fn sides_are_independent() {
    let mut parser = Parser::new();
    parser.update(Side::C2S, &frame(&handshake_body(2)));

    // Bytes on C2S must not be visible on S2C.
    assert!(parser.try_parse(Side::S2C).unwrap().is_none());
    assert!(!parser.is_empty(Side::C2S));
}
