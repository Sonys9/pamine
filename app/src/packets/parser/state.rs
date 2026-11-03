//! Connection state machine and the packet/side enums.

use std::sync::atomic::{AtomicU8, Ordering};

use crate::packets::{c2s::packets::C2SPacket, s2c::packets::S2CPacket};

/// Result of a single parse attempt.
///
/// `Ok(None)` means the buffer does not yet hold a complete packet — the caller
/// should read more bytes and retry. This is distinct from `Err`, which means
/// the stream is malformed.
pub type ParseResult<'a> = eyre::Result<Option<Packet<'a>>>;

/// Which side of the connection a packet belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Client to server.
    C2S,
    /// Server to client.
    S2C,
}

pub struct AtomicState {
    pub state: AtomicU8,
}

impl AtomicState {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(u8::try_from(State::Handshake).unwrap()),
        }
    }
    pub fn get(&self) -> State {
        State::try_from(self.state.load(Ordering::Acquire)).unwrap()
    }
    pub fn set(&self, state: State) {
        self.state
            .store(u8::try_from(state).unwrap(), Ordering::Release);
    }
}

/// Protocol phase. Advanced by `Handshake` and `Login Success`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Handshake,
    Login,
    Play,
    Status,
}

impl TryFrom<u8> for State {
    type Error = eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => State::Handshake,
            1 => State::Login,
            2 => State::Play,
            3 => State::Status,
            _ => eyre::bail!("Bad state id"),
        })
    }
}

impl TryFrom<State> for u8 {
    type Error = eyre::Report;

    fn try_from(value: State) -> Result<Self, Self::Error> {
        Ok(match value {
            State::Handshake => 0,
            State::Login => 1,
            State::Play => 2,
            State::Status => 3,
        })
    }
}

/// A decoded packet, tagged with the direction it travelled.
#[derive(Clone, Debug)]
pub enum Packet<'a> {
    C2S(C2SPacket<'a>),
    S2C(S2CPacket<'a>),
}
