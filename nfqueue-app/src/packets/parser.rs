//! Minecraft packet framing and dispatch.
//!
//! A TCP stream carries a sequence of length-prefixed packets. Once the server
//! sends `Set Compression`, every packet gains an extra `data_len` field and its
//! body may be zlib-compressed.
//!
//! ```text
//! uncompressed: len:VarInt  body[len]
//! compressed:   len:VarInt  data_len:VarInt  body
//! ```
//!
//! `data_len == 0` means the body is stored uncompressed even though compression
//! is enabled.

use std::io::Read;

use flate2::read::ZlibDecoder;
use pcore::decoder::Decoder;

use crate::{
    buffer::BigBuffer,
    packets::{
        c2s::{self, packets::C2SPacket},
        s2c::{self, packets::S2CPacket},
    },
};

/// Result of a single parse attempt.
///
/// `Ok(None)` means the buffer does not yet hold a complete packet — the caller
/// should read more bytes and retry. This is distinct from `Err`, which means
/// the stream is malformed.
pub type ParseResult<'a> = eyre::Result<Option<Packet<'a>>>;

pub struct Parser {
    state: State,
    c2s: StreamData,
    s2c: StreamData,
    /// `Some(threshold)` once compression is enabled; `None` while disabled.
    threshold: Option<i32>,
    /// Scratch space for zlib output, reused across packets.
    decompress_buffer: Vec<u8>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Handshake,
            c2s: StreamData::default(),
            s2c: StreamData::default(),
            threshold: None,
            decompress_buffer: Vec::new(),
        }
    }

    /// Appends freshly received bytes for one direction.
    pub fn update(&mut self, side: Side, bytes: &[u8]) {
        self.stream(side).buffer.update(bytes, bytes.len());
    }

    /// Parses at most one packet from the front of the buffer.
    ///
    /// On success the read cursor advances by exactly one packet, so calling
    /// this in a loop drains every complete packet in the buffer.
    pub fn try_parse<'a>(&'a mut self, side: Side) -> ParseResult<'a> {
        let threshold = self.threshold;

        // Borrow the three fields we need separately: the receive buffer
        // (shared), the scratch buffer (mutable) and the state machine
        // (mutable). Going through a `&mut self` helper would borrow all of
        // `self` at once and the borrow checker would reject the split.
        let stream = match side {
            Side::C2S => &mut self.c2s,
            Side::S2C => &mut self.s2c,
        };
        let pending: &'a [u8] = &stream.buffer.buffer[stream.diff..stream.buffer.len];

        let Some(frame) = Frame::split(pending, threshold)? else {
            return Ok(None);
        };
        let consumed = frame.consumed;

        let body: &'a [u8] = if frame.compressed {
            let scratch = &mut self.decompress_buffer;
            scratch.clear();
            let written = inflate(frame.body, scratch)?;
            &scratch[..written]
        } else {
            frame.body
        };

        let mut decoder = Decoder::new(body);
        let opcode = decoder.varint()?.get();

        // Advance only after the frame was fully validated.
        stream.diff += consumed;

        let packet = match self.state {
            State::Handshake => Self::handshake_side(&mut self.state, opcode, decoder, side)?,
            State::Login => {
                Self::login_side(&mut self.state, opcode, decoder, side, &mut self.threshold)?
            }
            State::Status => Self::status_side(opcode, decoder, side)?,
            State::Play => Self::play_side(opcode, decoder, side)?,
        };
        Ok(Some(packet))
    }

    /// Drops already-parsed bytes and rewinds the read cursor.
    pub fn clear(&mut self, side: Side) {
        let stream = self.stream(side);
        stream.buffer.move_buffer(stream.diff, 0);
        stream.diff = 0;
    }

    /// True when no unparsed bytes remain for this direction.
    ///
    /// Note this is not the same as "the buffer is empty": parsed bytes stay in
    /// the buffer until [`clear`](Self::clear) compacts it.
    pub fn is_empty(&self, side: Side) -> bool {
        let stream = self.stream_ref(side);
        stream.diff >= stream.buffer.len
    }

    /// Resets the connection state machine and both buffers.
    pub fn reset(&mut self) {
        self.state = State::Handshake;
        self.threshold = None;
        self.decompress_buffer.clear();
        self.c2s = StreamData::default();
        self.s2c = StreamData::default();
    }

    pub fn state(&self) -> State {
        self.state
    }

    fn stream(&mut self, side: Side) -> &mut StreamData {
        match side {
            Side::C2S => &mut self.c2s,
            Side::S2C => &mut self.s2c,
        }
    }

    fn stream_ref(&self, side: Side) -> &StreamData {
        match side {
            Side::C2S => &self.c2s,
            Side::S2C => &self.s2c,
        }
    }

    fn handshake_side<'a>(
        state: &mut State,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        let Side::C2S = side else {
            eyre::bail!("server sent a handshake packet");
        };

        let parsed = c2s::handshake::Handshake::decode_by_opcode(opcode, &mut decoder)?;
        let c2s::handshake::Handshake::Handshaking(ref packet) = parsed;
        *state = match packet.next_state.get() {
            1 => State::Status,
            _ => State::Login,
        };
        Ok(Packet::C2S(C2SPacket::Handshake(parsed)))
    }

    fn login_side<'a>(
        state: &mut State,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
        threshold: &mut Option<i32>,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Login(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                match &parsed {
                    s2c::login::Login::LoginSuccess(_) => *state = State::Play,
                    s2c::login::Login::SetCompression(set) => {
                        let value = set.threshold.get();
                        // A negative threshold disables compression entirely.
                        *threshold = (value >= 0).then_some(value);
                    }
                    _ => {}
                }
                Ok(Packet::S2C(S2CPacket::Login(parsed)))
            }
        }
    }

    fn status_side<'a>(
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::status::Status::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Status(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::status::Status::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::S2C(S2CPacket::Status(parsed)))
            }
        }
    }

    fn play_side<'a>(
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::play::Play::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Play(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::play::Play::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::S2C(S2CPacket::Play(parsed)))
            }
        }
    }
}

/// One framed packet, still pointing into the receive buffer.
struct Frame<'a> {
    /// Packet body, positioned at the opcode.
    body: &'a [u8],
    /// Whether `body` still needs zlib decompression.
    compressed: bool,
    /// Total bytes this frame occupies in the receive buffer, length prefix
    /// included. This is what the read cursor must advance by.
    consumed: usize,
}

impl<'a> Frame<'a> {
    /// Splits the first frame off `pending`.
    ///
    /// Returns `Ok(None)` when `pending` does not yet contain a whole frame.
    fn split(pending: &'a [u8], threshold: Option<i32>) -> eyre::Result<Option<Self>> {
        let mut head = Decoder::new(pending);
        let Ok(len) = head.varint().map(|v| v.get()) else {
            // Not even the length prefix is complete yet.
            return Ok(None);
        };
        if len < 0 {
            eyre::bail!("negative packet length: {len}");
        }
        let len = len as usize;
        let prefix_len = head.len();
        let total = len
            .checked_add(prefix_len)
            .ok_or_else(|| eyre::eyre!("packet length overflow"))?;

        if pending.len() < total {
            return Ok(None);
        }

        let body = &pending[prefix_len..total];

        let Some(_threshold) = threshold else {
            return Ok(Some(Frame {
                body,
                compressed: false,
                consumed: total,
            }));
        };

        // Compression is on: the body starts with the uncompressed length.
        let mut inner = Decoder::new(body);
        let data_len = inner.varint()?.get();
        if data_len <= 0 {
            // Stored uncompressed despite compression being enabled.
            return Ok(Some(Frame {
                body: inner.get(),
                compressed: false,
                consumed: total,
            }));
        }
        Ok(Some(Frame {
            body: inner.get(),
            compressed: true,
            consumed: total,
        }))
    }
}

fn inflate(payload: &[u8], out: &mut Vec<u8>) -> eyre::Result<usize> {
    let mut decoder = ZlibDecoder::new(payload);
    Ok(decoder.read_to_end(out)?)
}

pub struct StreamData {
    pub buffer: BigBuffer,
    /// Offset of the first unparsed byte.
    diff: usize,
}

impl Default for StreamData {
    fn default() -> Self {
        Self {
            buffer: BigBuffer::new(),
            diff: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Handshake,
    Login,
    Play,
    Status,
}

#[derive(Clone, Debug)]
pub enum Packet<'a> {
    C2S(C2SPacket<'a>),
    S2C(S2CPacket<'a>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    C2S,
    S2C,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcore::encoder::varint;

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
}
