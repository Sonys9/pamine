//! Parser tests: framing, fragmentation and state transitions.

use pcore::{Decode, encode, encoder::varint};

use super::{
    Packet, Parsed, Parser, Side, State, UnsureThreshold,
    frame::{Frame, MAX_PACKET_LEN, deflate, inflate},
    frame_packet,
};
use crate::{
    connections::meta::Meta,
    packets::{
        c2s::{
            handshake::{Handshake, Handshaking},
            packets::C2SPacket,
            play::{ChatCommand, ChatMessage, Play},
        },
        s2c::packets::S2CPacket,
    },
};

/// Builds an uncompressed frame: `len:VarInt body`.
fn frame(body: &[u8]) -> Vec<u8> {
    let mut out = varint(body.len() as i32);
    out.extend_from_slice(body);
    out
}

/// A threshold that is still unset, i.e. compression is disabled.
fn uncompressed() -> UnsureThreshold {
    UnsureThreshold::new()
}

/// A threshold that has been set to `n`.
fn compressed(n: i32) -> UnsureThreshold {
    let threshold = UnsureThreshold::new();
    threshold.set(n);
    threshold
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
    let parser = Parser::new();
    let mut meta = Meta::new();
    meta.stream_data.update(&frame(&handshake_body(2))).unwrap();

    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Packet { .. })
    ));
    assert_eq!(parser.state(), State::Login);
    assert!(meta.stream_data.is_empty());
}

#[test]
fn parses_two_packets_from_one_read() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    // Handshake with next_state=1 moves us to Status, where the state does
    // not change again, so both following packets parse the same way.
    meta.stream_data.update(&frame(&handshake_body(1))).unwrap();
    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Packet { .. })
    ));
    meta.stream_data.clear();
    assert_eq!(parser.state(), State::Status);

    // Two Status Request packets (opcode 0x00, no fields) in one read.
    let mut bytes = frame(&[0x00]);
    bytes.extend_from_slice(&frame(&[0x00]));
    meta.stream_data.update(&bytes).unwrap();

    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Packet { .. })
    ));
    meta.stream_data.clear();
    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Packet { .. })
    ));
    meta.stream_data.clear();
    assert!(meta.stream_data.is_empty());
}

#[test]
fn incomplete_packet_returns_none() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    let full = frame(&handshake_body(2));
    meta.stream_data.update(&full[..full.len() - 1]).unwrap();

    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Incomplete)
    ));
    // Nothing was consumed, so the bytes are still there.
    assert!(!meta.stream_data.is_empty());
}

#[test]
fn empty_buffer_returns_none() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Incomplete)
    ));
}

#[test]
fn fragmented_packet_completes_after_second_read() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    let full = frame(&handshake_body(2));
    let (a, b) = full.split_at(5);

    meta.stream_data.update(a).unwrap();
    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Incomplete)
    ));

    meta.stream_data.update(b).unwrap();
    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Packet { .. })
    ));
}

#[test]
fn negative_length_is_an_error() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    meta.stream_data.update(&varint(-1)).unwrap();
    assert!(parser.try_parse(Side::C2S, &mut meta).is_err());
}

#[test]
fn reset_restores_initial_state() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    meta.stream_data.update(&frame(&handshake_body(2))).unwrap();
    parser.try_parse(Side::C2S, &mut meta).unwrap();
    assert_eq!(parser.state(), State::Login);

    parser.reset(&mut meta);
    assert_eq!(parser.state(), State::Handshake);
    assert!(meta.stream_data.is_empty());
}

#[test]
fn sides_are_independent() {
    let parser = Parser::new();
    let mut c2s = Meta::new();
    let mut s2c = Meta::new();
    c2s.stream_data.update(&frame(&handshake_body(2))).unwrap();

    // Bytes on C2S must not be visible on S2C.
    assert!(matches!(
        parser.try_parse(Side::S2C, &mut s2c),
        Ok(Parsed::Incomplete)
    ));
    assert!(!c2s.stream_data.is_empty());
}

#[test]
fn frame_packet_without_compression_is_a_plain_frame() {
    let body = handshake_body(2);
    let framed = frame_packet(&body, &uncompressed()).unwrap();
    assert_eq!(framed, frame(&body));
}

#[test]
fn frame_packet_below_threshold_stores_raw_with_zero_data_len() {
    let body = handshake_body(2);
    let framed = frame_packet(&body, &compressed(256)).unwrap();

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
    let threshold = compressed(256);
    let framed = frame_packet(&body, &threshold).unwrap();

    // The frame must be readable by the parser's own splitter.
    let split = Frame::split(&framed, &threshold).unwrap().unwrap();
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
    let compressed_body = deflate(&bomb).unwrap();

    let mut out = Vec::new();
    let err = inflate(&compressed_body, &mut out, 16).unwrap_err();
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
    let compressed_body = deflate(&bomb).unwrap();
    assert!(
        compressed_body.len() < 16 * 1024,
        "the bomb should be small on the wire, got {} bytes",
        compressed_body.len()
    );

    let mut out = Vec::new();
    let err = inflate(&compressed_body, &mut out, 1000).unwrap_err();
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
    let compressed_body = deflate(&body).unwrap();

    let mut out = Vec::new();
    let written = inflate(&compressed_body, &mut out, body.len()).unwrap();
    assert_eq!(written, body.len());
    assert_eq!(out, body);
}

#[test]
fn inflate_rejects_a_stream_shorter_than_data_len() {
    let body = b"hello";
    let compressed_body = deflate(body).unwrap();

    let mut out = Vec::new();
    let err = inflate(&compressed_body, &mut out, 100).unwrap_err();
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
    let err = match Frame::split(&framed, &uncompressed()) {
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

    let err = match Frame::split(&framed, &compressed(256)) {
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
    let parser = Parser::new();
    let mut meta = Meta::new();
    let body = handshake_body(2);
    let framed = frame_packet(&body, &uncompressed()).unwrap();
    meta.stream_data.update(&framed).unwrap();

    assert!(matches!(
        parser.try_parse(Side::C2S, &mut meta),
        Ok(Parsed::Packet { .. })
    ));
    assert_eq!(parser.state(), State::Login);
    assert!(meta.stream_data.is_empty());
}

#[test]
fn injected_frame_parses_without_touching_the_receive_buffer() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    let mut scratch = Vec::new();

    // Real traffic already sits in the buffer; the injection must not disturb it.
    let real = frame(&handshake_body(2));
    meta.stream_data.update(&real).unwrap();

    let injected = frame_packet(&handshake_body(2), &uncompressed()).unwrap();
    let parsed = parser
        .parse_injected(Side::C2S, &injected, &mut scratch)
        .unwrap();

    assert!(matches!(parsed, Parsed::Packet { .. }));
    assert_eq!(parsed.consumed(), Some(injected.len()));
    // The receive buffer and its cursor are untouched.
    assert_eq!(meta.stream_data.diff, 0);
    assert_eq!(meta.stream_data.pending(), &real[..]);
}

#[test]
fn injected_frame_advances_the_state_machine() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    let mut scratch = Vec::new();

    let injected = frame_packet(&handshake_body(2), &uncompressed()).unwrap();
    parser
        .parse_injected(Side::C2S, &injected, &mut scratch)
        .unwrap();

    // An injected handshake is still a handshake: the state must move.
    assert_eq!(parser.state(), State::Login);
}

#[test]
fn injected_partial_frame_is_incomplete() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    let mut scratch = Vec::new();

    let injected = frame_packet(&handshake_body(2), &uncompressed()).unwrap();
    let parsed = parser
        .parse_injected(Side::C2S, &injected[..injected.len() - 1], &mut scratch)
        .unwrap();

    assert!(matches!(parsed, Parsed::Incomplete));
    assert_eq!(parsed.consumed(), None);
}

#[test]
fn injected_compressed_frame_is_decompressed() {
    let parser = Parser::new();
    let mut meta = Meta::new();
    let mut scratch = Vec::new();

    // Enable compression on the parser, then build a frame that is large
    // enough to actually be compressed.
    parser.threshold().set(1);
    let body = handshake_body(2);
    let injected = frame_packet(&body, parser.threshold()).unwrap();

    let parsed = parser
        .parse_injected(Side::C2S, &injected, &mut scratch)
        .unwrap();

    assert!(matches!(parsed, Parsed::Packet { .. }));
    assert_eq!(parsed.consumed(), Some(injected.len()));
    assert_eq!(parser.state(), State::Login);
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
    let framed = frame_packet(&body, &uncompressed()).unwrap();

    let parser = Parser::new();
    let mut c2s = Meta::new();
    let mut s2c = Meta::new();

    // Skip the handshake: jump straight to Play by parsing a handshake first.
    c2s.stream_data.update(&frame(&handshake_body(2))).unwrap();
    parser.try_parse(Side::C2S, &mut c2s).unwrap();
    c2s.stream_data.clear();
    assert_eq!(parser.state(), State::Login);

    // `Login Success` is S2C, so drive the state machine from that side.
    let mut login_success = vec![0x02]; // opcode
    login_success.extend_from_slice(&[0u8; 16]); // uuid
    login_success.extend_from_slice(&[0]); // empty name
    login_success.extend_from_slice(&[0]); // no properties
    s2c.stream_data.update(&frame(&login_success)).unwrap();
    parser.try_parse(Side::S2C, &mut s2c).unwrap();
    s2c.stream_data.clear();
    assert_eq!(parser.state(), State::Play);

    c2s.stream_data.update(&framed).unwrap();
    {
        let parsed = parser.try_parse(Side::C2S, &mut c2s).unwrap();
        let Parsed::Packet { packet, .. } = parsed else {
            panic!("expected a packet, got {parsed:?}");
        };
        match packet {
            super::Packet::C2S(crate::packets::c2s::packets::C2SPacket::Play(
                Play::ChatMessage(m),
            )) => {
                assert_eq!(m.message, "!dlrow ,olleH");
            }
            other => panic!("unexpected packet: {other:?}"),
        }
    }
    assert!(c2s.stream_data.is_empty());
}

#[test]
fn packet_exposes_the_opcode_of_a_c2s_handshake() {
    // The whole direction/phase chain must forward `opcode()` down to the
    // variant's `#[id(N)]`, so a caller never has to match the layers by hand.
    let packet = Packet::C2S(C2SPacket::Handshake(Handshake::Handshaking(
        Handshaking {
            protocol: pcore::decoder::VarInt::new(760),
            address: "mc.server.com",
            port: 25565,
            next_state: pcore::decoder::VarInt::new(1),
        },
    )));

    assert_eq!(packet.opcode(), 0x00);
}

#[test]
fn packet_opcode_agrees_with_the_encoded_first_varint() {
    // `opcode()` is only useful if it is the same value `Encode` writes, so
    // compare it against the first VarInt of the encoded body for both sides.
    let c2s_play = Play::ChatMessage(chat_message("hi"));
    let s2c_status = crate::packets::s2c::status::Status::PingResponse(
        crate::packets::s2c::status::PingResponse { payload: 7 },
    );

    let c2s = Packet::C2S(C2SPacket::Play(c2s_play.clone()));
    let s2c = Packet::S2C(S2CPacket::Status(s2c_status.clone()));

    for (packet, inner) in [
        (&c2s, encode(&c2s_play)),
        (&s2c, encode(&s2c_status)),
    ] {
        let mut decoder = pcore::decoder::Decoder::new(&inner);
        let written = decoder.varint().unwrap().get();
        assert_eq!(written, packet.opcode(), "opcode() disagrees with Encode");
    }

    assert_eq!(c2s.opcode(), 0x05);
    assert_eq!(s2c.opcode(), 0x01);
}
