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
pub use state::{Parsed, Side};
pub use stream::StreamData;

// `Packet` is part of the public API surface of `ParseResult`, but nothing in
// this crate names it directly, so re-export it without tripping `unused_imports`.
#[allow(unused_imports)]
pub use state::Packet;

use pcore::decoder::Decoder;

use frame::{Frame, inflate};
use tracing::error;

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

    /// Parses at most one packet from the front of the receive buffer.
    ///
    /// On success the read cursor advances by exactly one packet, so calling
    /// this in a loop drains every complete packet in the buffer.
    ///
    /// # Errors
    ///
    /// Only *framing* errors are returned as `Err`: the next packet boundary
    /// cannot be found, so the stream is unrecoverable. A body that fails to
    /// decode comes back as [`Parsed::BodyError`] with the frame size, so the
    /// caller can skip it and keep going.
    pub fn try_parse<'a>(&'a self, side: Side, meta: &'a mut Meta) -> eyre::Result<Parsed<'a>> {
        // Touch the fields directly rather than through `StreamData::pending`:
        // a method call would borrow the whole struct, and we need `buffer`
        // (shared, for `'a`) and `diff` (mutable) at the same time.
        let pending =
            &meta.stream_data.buffer.buffer[meta.stream_data.diff..meta.stream_data.buffer.len];

        let Some(frame) = Frame::split(pending, &self.threshold)? else {
            return Ok(Parsed::Incomplete);
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

        // Advance only after the frame was fully validated. Assign the field
        // directly: `stream.advance(..)` would re-borrow the whole struct while
        // `pending` still holds a shared borrow of `stream.buffer`.
        meta.stream_data.diff += consumed;

        Ok(self.decode_body(side, body, &pending[..consumed], consumed))
    }

    /// Parses one injected frame.
    ///
    /// The frame comes from outside the connection, so it must not touch the
    /// receive buffer or its cursor. `scratch` is the caller's decompression
    /// buffer, used only when the frame is compressed.
    ///
    /// # Errors
    ///
    /// A framing error is returned as `Err`, same as [`Self::try_parse`]. An
    /// injected frame that does not form a complete packet is
    /// [`Parsed::Incomplete`], which the caller should treat as a bug in the
    /// injector rather than as "read more bytes".
    pub fn parse_injected<'a>(
        &'a self,
        side: Side,
        bytes: &'a [u8],
        scratch: &'a mut Vec<u8>,
    ) -> eyre::Result<Parsed<'a>> {
        let Some(frame) = Frame::split(bytes, &self.threshold)? else {
            return Ok(Parsed::Incomplete);
        };
        let consumed = frame.consumed;

        let body: &'a [u8] = if frame.compressed {
            scratch.clear();
            scratch.reserve(frame.data_len);
            let written = inflate(frame.body, scratch, frame.data_len)?;
            &scratch[..written]
        } else {
            frame.body
        };

        Ok(self.decode_body(side, body, &bytes[..consumed], consumed))
    }

    /// Shared tail of both parse paths: opcode, dispatch, error mapping.
    ///
    /// `frame` is the complete frame the body came from, carried through so the
    /// caller can forward it untouched without borrowing the receive buffer a
    /// second time.
    fn decode_body<'a>(
        &'a self,
        side: Side,
        body: &'a [u8],
        frame: &'a [u8],
        consumed: usize,
    ) -> Parsed<'a> {
        let mut decoder = Decoder::new(body);
        let opcode = match decoder.varint() {
            Ok(opcode) => opcode.get(),
            Err(error) => {
                return Parsed::BodyError {
                    error: error.into(),
                    frame,
                    consumed,
                };
            }
        };

        let state = self.state.get();
        let packet = match state {
            State::Handshake => dispatch::handshake(&self.state, opcode, decoder, side),
            State::Login => dispatch::login(&self.state, opcode, decoder, side, &self.threshold),
            State::Status => dispatch::status(opcode, decoder, side),
            State::Play => dispatch::play(opcode, decoder, side),
        };

        match packet {
            Ok(packet) => Parsed::Packet {
                packet,
                frame,
                consumed,
            },
            Err(error) => {
                // A body that fails to decode is not fatal: the frame boundary
                // is known, so the caller can skip it. Log the opcode so a
                // mis-modelled packet can be identified from the log alone.
                error!(
                    side = ?side,
                    state = ?state,
                    opcode = format_args!("0x{opcode:02X}"),
                    body_len = body.len(),
                    "failed to decode packet body: {error}"
                );
                Parsed::BodyError {
                    error,
                    frame,
                    consumed,
                }
            }
        }
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
