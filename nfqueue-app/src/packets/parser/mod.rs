//! Minecraft packet framing and dispatch.
//!
//! A TCP stream carries a sequence of length-prefixed packets. Once the server
//! sends `Set Compression`, every packet gains an extra `data_len` field and its
//! body may be zlib-compressed.
//!
//! Module layout:
//!
//! - [`state`] — protocol phases, [`Side`], [`Packet`], [`ParseResult`]
//! - [`stream`] — per-direction receive buffer and read cursor
//! - [`frame`] — length prefix, compression header, zlib payload
//! - [`dispatch`] — per-state packet decoding and state transitions
//!
//! [`Side`]: state::Side
//! [`Packet`]: state::Packet
//! [`ParseResult`]: state::ParseResult

mod dispatch;
pub mod frame;
mod state;
mod stream;

#[cfg(test)]
mod tests;

pub use frame::frame_packet;
pub use state::{ParseResult, Side, State};
pub use stream::StreamData;

// `Packet` is part of the public API surface of `ParseResult`, but nothing in
// this crate names it directly, so re-export it without tripping `unused_imports`.
#[allow(unused_imports)]
pub use state::Packet;

use pcore::decoder::Decoder;

use frame::{Frame, inflate};

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
        self.stream_mut(side).buffer.update(bytes, bytes.len());
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
        // Touch the fields directly rather than through `StreamData::pending`:
        // a method call would borrow the whole struct, and we need `buffer`
        // (shared, for `'a`) and `diff` (mutable) at the same time.
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

        // Advance only after the frame was fully validated. Assign the field
        // directly: `stream.advance(..)` would re-borrow the whole struct while
        // `pending` still holds a shared borrow of `stream.buffer`.
        stream.diff += consumed;

        let packet = match self.state {
            State::Handshake => dispatch::handshake(&mut self.state, opcode, decoder, side)?,
            State::Login => {
                dispatch::login(&mut self.state, opcode, decoder, side, &mut self.threshold)?
            }
            State::Status => dispatch::status(opcode, decoder, side)?,
            State::Play => dispatch::play(opcode, decoder, side)?,
        };
        Ok(Some(packet))
    }

    
    /// Returns parsed packet's bytes
    pub fn packet_bytes(&mut self, side: Side) -> &[u8] {
        let stream_data = self.stream_mut(side);
        &stream_data.buffer.buffer[..stream_data.diff]
    }

    /// The compression threshold currently in effect.
    ///
    /// `None` while compression is disabled. Needed by callers that build a
    /// packet themselves: a hand-made frame must carry the same `data_len`
    /// header the peer expects.
    pub fn threshold(&self) -> Option<i32> {
        self.threshold
    }
    
    /// Drops already-parsed bytes and rewinds the read cursor.
    pub fn clear(&mut self, side: Side) {
        self.stream_mut(side).compact();
    }

    /// True when no unparsed bytes remain for this direction.
    ///
    /// Note this is not the same as "the buffer is empty": parsed bytes stay in
    /// the buffer until [`clear`](Self::clear) compacts it.
    pub fn is_empty(&self, side: Side) -> bool {
        self.stream(side).is_drained()
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

    fn stream(&self, side: Side) -> &StreamData {
        match side {
            Side::C2S => &self.c2s,
            Side::S2C => &self.s2c,
        }
    }

    fn stream_mut(&mut self, side: Side) -> &mut StreamData {
        match side {
            Side::C2S => &mut self.c2s,
            Side::S2C => &mut self.s2c,
        }
    }
}
