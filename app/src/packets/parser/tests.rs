//! Parser tests: framing, fragmentation and state transitions.

use pcore::{Decode, encode, encoder::varint};

use super::{
    Parser, Side, State,
    frame::{Frame, MAX_PACKET_LEN, deflate, inflate},
    frame_packet,
};
use crate::packets::c2s::play::{ChatCommand, ChatMessage, Play};

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

#[test]
fn frame_packet_without_compression_is_a_plain_frame() {
    let body = handshake_body(2);
    let framed = frame_packet(&body, None).unwrap();
    assert_eq!(framed, frame(&body));
}

#[test]
fn frame_packet_below_threshold_stores_raw_with_zero_data_len() {
    let body = handshake_body(2);
    let framed = frame_packet(&body, Some(256)).unwrap();

    // `len:VarInt data_len:VarInt body`, with `data_len == 0`.
    let mut expected = varint((body.len() + 1) as i32);
    expected.push(0);
    expected.extend_from_slice(&body);
    assert_eq!(framed, expected);
}

#[test]
fn frame_packet_above_threshold_compresses_and_round_trips() {
    // A body large enough to be compressed.
    let body = vec![0x42u8; 512];
    let framed = frame_packet(&body, Some(256)).unwrap();

    // The frame must be readable by the parser's own splitter.
    let split = Frame::split(&framed, Some(256)).unwrap().unwrap();
    assert!(split.compressed);
    assert_eq!(split.consumed, framed.len());
    assert_eq!(split.data_len, body.len(), "data_len must be the raw size");

    let mut out = Vec::new();
    let written = inflate(split.body, &mut out, split.data_len).unwrap();
    assert_eq!(written, body.len());
    assert_eq!(out, body);
}

#[test]
fn inflate_rejects_a_stream_larger_than_data_len() {
    // A zlib bomb: 64 KiB of zeros compresses to a few hundred bytes. If the
    // header claims 16 bytes, the decompressor must stop instead of expanding
    // the whole thing.
    let bomb = vec![0u8; 64 * 1024];
    let compressed = deflate(&bomb).unwrap();

    let mut out = Vec::new();
    let err = inflate(&compressed, &mut out, 16).unwrap_err();
    assert!(
        err.to_string().contains("data_len declared 16"),
        "unexpected error: {err}"
    );
    // The cap is `expected + 1`, so at most 17 bytes were written.
    assert!(out.len() <= 17, "wrote {} bytes", out.len());
}

#[test]
fn inflate_stops_a_bomb_that_claims_a_tiny_size() {
    // The attack: a frame declares `data_len = 1000` but the zlib stream
    // expands to 8 MiB. Without the cap, `read_to_end` would allocate all 8 MiB
    // (and a real bomb could claim gigabytes). With it, the reader stops at
    // 1001 bytes and the mismatch is reported.
    let bomb = vec![0u8; 8 * 1024 * 1024];
    let compressed = deflate(&bomb).unwrap();
    assert!(
        compressed.len() < 16 * 1024,
        "the bomb should be small on the wire, got {} bytes",
        compressed.len()
    );

    let mut out = Vec::new();
    let err = inflate(&compressed, &mut out, 1000).unwrap_err();
    assert!(
        err.to_string().contains("data_len declared 1000"),
        "unexpected error: {err}"
    );

    // The whole point: memory stayed bounded by the declared size, not by the
    // 8 MiB the stream actually contains.
    assert!(out.len() <= 1001, "wrote {} bytes", out.len());
    assert!(
        out.capacity() <= 4096,
        "allocated {} bytes for a 1000-byte claim",
        out.capacity()
    );
}

#[test]
fn inflate_accepts_a_stream_that_matches_data_len() {
    // The happy path must still work: an honest header is not rejected.
    let body = vec![0xABu8; 4096];
    let compressed = deflate(&body).unwrap();

    let mut out = Vec::new();
    let written = inflate(&compressed, &mut out, body.len()).unwrap();
    assert_eq!(written, body.len());
    assert_eq!(out, body);
}

#[test]
fn inflate_rejects_a_stream_shorter_than_data_len() {
    let body = b"hello";
    let compressed = deflate(body).unwrap();

    let mut out = Vec::new();
    let err = inflate(&compressed, &mut out, 100).unwrap_err();
    assert!(
        err.to_string().contains("data_len declared 100"),
        "unexpected error: {err}"
    );
}

#[test]
fn split_rejects_an_oversized_frame_length() {
    // A length prefix above the limit must fail before the bytes arrive.
    let mut framed = varint((MAX_PACKET_LEN + 1) as i32);
    framed.push(0x00);
    let err = match Frame::split(&framed, None) {
        Err(e) => e,
        Ok(_) => panic!("oversized frame length was accepted"),
    };
    assert!(
        err.to_string().contains("exceeds the"),
        "unexpected error: {err}"
    );
}

#[test]
fn split_rejects_an_oversized_data_len() {
    // `len` is small, but the declared uncompressed size is not.
    let mut inner = varint((MAX_PACKET_LEN + 1) as i32);
    inner.extend_from_slice(&[0x00; 4]);
    let mut framed = varint(inner.len() as i32);
    framed.extend_from_slice(&inner);

    let err = match Frame::split(&framed, Some(256)) {
        Err(e) => e,
        Ok(_) => panic!("oversized data_len was accepted"),
    };
    assert!(
        err.to_string().contains("data_len"),
        "unexpected error: {err}"
    );
}

#[test]
fn frame_packet_round_trips_through_the_parser() {
    let mut parser = Parser::new();
    let body = handshake_body(2);
    let framed = frame_packet(&body, None).unwrap();
    parser.update(Side::C2S, &framed);

    assert!(parser.try_parse(Side::C2S).unwrap().is_some());
    assert_eq!(parser.state(), State::Login);
    assert!(parser.is_empty(Side::C2S));
}

/// A `Chat Message` body with no signature and no last-seen entries.
fn chat_message(text: &str) -> ChatMessage<'_> {
    ChatMessage {
        message: text,
        timestamp: 1,
        salt: 2,
        signature: None,
        signed_preview: false,
        last_seen_messages: Vec::new(),
        last_received: None,
    }
}

#[test]
fn encoding_the_enum_writes_the_opcode_from_the_id_attribute() {
    // The whole point of encoding `Play` instead of `ChatMessage`: the opcode
    // comes from `#[id(0x05)]` and is never spelled out by hand.
    let packet = Play::ChatMessage(chat_message("hi"));
    assert_eq!(packet.opcode(), 0x05);

    let bytes = encode(&packet);
    assert_eq!(bytes[0], 0x05, "opcode must be the first byte");
}

#[test]
fn chat_command_tail_is_written_without_a_length_prefix() {
    // The `last_seen_messages` tail is raw bytes, not a VarInt-prefixed array.
    // A prefix here would add a byte and the server would read past the end of
    // the packet (`readerIndex(22) + length(1) exceeds writerIndex(22)`).
    let command = ChatCommand {
        command: "?",
        timestamp: 1,
        salt: 2,
        argument_signatures: Vec::new(),
        signed_preview: false,
        last_seen_messages: &[0, 0],
    };
    let bytes = encode(&Play::ChatCommand(command));

    // opcode(1) + strlen(1) + "?"(1) + ts(8) + salt(8) + count(1) + preview(1)
    // + tail(2) = 23. A length prefix on the tail would make it 24.
    assert_eq!(bytes.len(), 23, "unexpected body length: {bytes:?}");
    assert_eq!(&bytes[bytes.len() - 2..], &[0, 0], "tail must be verbatim");
}

#[test]
fn chat_command_round_trips_through_decode() {
    // Decoding must consume the tail too, otherwise the next packet in the
    // stream would start two bytes early.
    let command = ChatCommand {
        command: "cmd",
        timestamp: 1793416506394,
        salt: -2348666467906564851,
        argument_signatures: Vec::new(),
        signed_preview: false,
        last_seen_messages: &[0, 0],
    };
    let bytes = encode(&Play::ChatCommand(command));

    let mut decoder = pcore::decoder::Decoder::new(&bytes);
    let opcode = decoder.varint().unwrap().get();
    assert_eq!(opcode, 0x04);
    let decoded = ChatCommand::decode(&mut decoder).unwrap();

    assert_eq!(decoded.command, "cmd");
    assert_eq!(decoded.last_seen_messages, &[0, 0]);
    assert_eq!(decoder.remaining(), 0, "tail was not consumed");
}

#[test]
fn spoofed_chat_message_survives_a_full_parse_round_trip() {
    // Reproduces the injection path: build the enum, encode it, frame it, then
    // feed the frame back through the parser. Before the fix the body was sent
    // without an opcode or a length prefix and the peer dropped the connection.
    let packet = Play::ChatMessage(chat_message("!dlrow ,olleH"));
    let body = encode(&packet);
    let framed = frame_packet(&body, None).unwrap();

    let mut parser = Parser::new();
    // Skip the handshake: jump straight to Play by parsing a handshake first.
    parser.update(Side::C2S, &frame(&handshake_body(2)));
    parser.try_parse(Side::C2S).unwrap();
    parser.clear(Side::C2S);
    assert_eq!(parser.state(), State::Login);

    // `Login Success` is S2C, so drive the state machine from that side.
    let mut login_success = vec![0x02]; // opcode
    login_success.extend_from_slice(&[0u8; 16]); // uuid
    login_success.extend_from_slice(&[0]); // empty name
    login_success.extend_from_slice(&[0]); // no properties
    parser.update(Side::S2C, &frame(&login_success));
    parser.try_parse(Side::S2C).unwrap();
    parser.clear(Side::S2C);
    assert_eq!(parser.state(), State::Play);

    parser.update(Side::C2S, &framed);
    let parsed = parser.try_parse(Side::C2S).unwrap().expect("packet");
    match parsed {
        super::Packet::C2S(crate::packets::c2s::packets::C2SPacket::Play(Play::ChatMessage(m))) => {
            assert_eq!(m.message, "!dlrow ,olleH");
        }
        other => panic!("unexpected packet: {other:?}"),
    }
    assert!(parser.is_empty(Side::C2S));
}
