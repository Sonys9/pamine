//! Connection state machine and the packet/side enums.

use crate::packets::{c2s::packets::C2SPacket, s2c::packets::S2CPacket};

/// Result of a single parse attempt.
///
/// `Ok(None)` means the buffer does not yet hold a complete packet — the caller
/// should read more bytes and retry. This is distinct from `Err`, which means
/// the stream is malformed.
pub type ParseResult = eyre::Result<Option<Packet>>;

/// Which side of the connection a packet belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Client to server.
    C2S,
    /// Server to client.
    S2C,
}

/// Protocol phase. Advanced by `Handshake` and `Login Success`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Handshake,
    Login,
    Play,
    Status,
}

/// A decoded packet, tagged with the direction it travelled.
#[derive(Clone, Debug)]
pub enum Packet {
    C2S(C2SPacket),
    S2C(S2CPacket),
}
