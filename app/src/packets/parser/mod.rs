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
mod threshold;

#[cfg(test)]
mod tests;

pub use frame::frame_packet;
pub use state::{ParseResult, Side};
pub use stream::StreamData;

// `Packet` is part of the public API surface of `ParseResult`, but nothing in
// this crate names it directly, so re-export it without tripping `unused_imports`.
#[allow(unused_imports)]
pub use state::Packet;

use pcore::decoder::Decoder;

use frame::{Frame, inflate};

use crate::connections::meta::Meta;
pub use crate::packets::parser::{
    state::{AtomicState, State},
    threshold::UnsureThreshold,
};

// Parser was holding StreamData, it's a bad way
pub struct Parser {
    state: AtomicState,
    /// `Some(threshold)` once compression is enabled; `None` while disabled.
    threshold: UnsureThreshold,
    // /// Scratch space for zlib output, reused across packets.
    // decompress_buffer: Vec<u8>,
    // Hmm it's better to make 2 decompress buffers I think!
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: AtomicState::new(),
            threshold: UnsureThreshold::new(),
        }
    }

    /// Parses at most one packet from the front of the buffer.
    ///
    /// On success the read cursor advances by exactly one packet, so calling
    /// this in a loop drains every complete packet in the buffer.
    pub fn try_parse<'a>(&'a self, side: Side, meta: &'a mut Meta) -> ParseResult<'a> {
        let threshold = &self.threshold;

        // Borrow the three fields we need separately: the receive buffer
        // (shared), the scratch buffer (mutable) and the state machine
        // (mutable). Going through a `&mut self` helper would borrow all of
        // `self` at once and the borrow checker would reject the split.
        // Touch the fields directly rather than through `StreamData::pending`:
        // a method call would borrow the whole struct, and we need `buffer`
        // (shared, for `'a`) and `diff` (mutable) at the same time.
        let pending: &'a [u8] =
            &meta.stream_data.buffer.buffer[meta.stream_data.diff..meta.stream_data.buffer.len];

        let Some(frame) = Frame::split(pending, threshold)? else {
            return Ok(None);
        };
        let consumed = frame.consumed;

        let body: &'a [u8] = if frame.compressed {
            let scratch = &mut meta.decompress_buffer;
            scratch.clear();
            // `data_len` was already bounded by `Frame::split`, so this cannot
            // reserve more than `MAX_PACKET_LEN` up front.
            scratch.reserve(frame.data_len);
            let written = inflate(frame.body, scratch, frame.data_len)?;
            &scratch[..written]
        } else {
            frame.body
        };

        let mut decoder = Decoder::new(body);
        let opcode = decoder.varint()?.get();

        // Advance only after the frame was fully validated. Assign the field
        // directly: `stream.advance(..)` would re-borrow the whole struct while
        // `pending` still holds a shared borrow of `stream.buffer`.
        meta.stream_data.diff += consumed;

        let state = self.state.get();
        let packet = match state {
            State::Handshake => dispatch::handshake(&self.state, opcode, decoder, side),
            State::Login => dispatch::login(&self.state, opcode, decoder, side, &self.threshold),
            State::Status => dispatch::status(opcode, decoder, side),
            State::Play => dispatch::play(opcode, decoder, side),
        };

        // A body that fails to decode is not fatal: the frame boundary is
        // already known, so the relay forwards the packet untouched. Log the
        // opcode so a mis-modelled packet can be identified from the log alone.
        let packet = match packet {
            Ok(packet) => packet,
            Err(e) => {
                error!(
                    side = ?side,
                    state = ?state,
                    opcode = format_args!("0x{opcode:02X}"),
                    body_len = body.len(),
                    "failed to decode packet body: {e}"
                );
                return Err(e);
            }
        };
        Ok(Some(packet))
    }

    /// The compression threshold currently in effect.
    ///
    /// `None` while compression is disabled. Needed by callers that build a
    /// packet themselves: a hand-made frame must carry the same `data_len`
    /// header the peer expects.
    pub fn threshold(&self) -> &UnsureThreshold {
        &self.threshold
    }

    /// Resets the connection state machine and the per-direction buffers.
    ///
    /// Takes `&self` because the state and threshold are atomics: nothing here
    /// needs exclusive access to the parser itself.
    pub fn reset(&self, meta: &mut Meta) {
        self.state.set(State::Handshake);
        self.threshold.reset();
        meta.decompress_buffer.clear();
        meta.stream_data = StreamData::default();
    }

    pub fn state(&self) -> State {
        self.state.get()
    }
}
