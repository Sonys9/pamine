//! Connection state machine and the packet/side enums.

use std::sync::atomic::{AtomicU8, Ordering};

use pcore_gen::Lua;

use crate::packets::{c2s::packets::C2SPacket, s2c::packets::S2CPacket};

/// Outcome of a single parse attempt.
///
/// A *framing* error is returned as `Err` by the parser: the next packet
/// boundary cannot be found, so the stream is unrecoverable. Everything else is
/// one of these three cases.
#[derive(Debug)]
pub enum Parsed<'a> {
    /// The input does not hold a complete frame yet. Read more and retry.
    Incomplete,
    /// A frame was decoded successfully.
    Packet {
        packet: Packet<'a>,
        /// The complete frame, `len:VarInt [data_len:VarInt] body`, ready to be
        /// forwarded as-is when the packet is not rewritten.
        ///
        /// Carried here so the caller never has to borrow the receive buffer a
        /// second time: `try_parse` already holds it for the packet's lifetime.
        frame: &'a [u8],
        /// How many bytes of the input this frame occupied.
        consumed: usize,
    },
    /// The frame was valid but its body failed to decode.
    ///
    /// `consumed` is still known, so the caller can skip the frame and keep the
    /// stream in sync instead of tearing the connection down. This matters
    /// because the decoder is not complete: an unknown or mis-modelled packet
    /// must not kill a working connection.
    BodyError {
        error: eyre::Report,
        /// The complete frame, forwarded untouched.
        frame: &'a [u8],
        consumed: usize,
    },
}

impl Parsed<'_> {
    /// How many bytes of the input this attempt consumed, if a frame was found.
    pub fn consumed(&self) -> Option<usize> {
        match self {
            Self::Incomplete => None,
            Self::Packet { consumed, .. } | Self::BodyError { consumed, .. } => Some(*consumed),
        }
    }
}

/// Which side of the connection a packet belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Client to server.
    C2S,
    /// Server to client.
    S2C,
}

impl TryFrom<Side> for u8 {
    type Error = ();

    fn try_from(value: Side) -> Result<Self, Self::Error> {
        Ok(match value {
            Side::C2S => 0,
            Side::S2C => 1,
        })
    }
}

impl TryFrom<u8> for Side {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Side::C2S,
            1 => Side::S2C,
            _ => return Err("Bad side id"),
        })
    }
}

impl TryFrom<&str> for Side {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "C2S" => Side::C2S,
            "S2C" => Side::S2C,
            _ => return Err("Bad side"),
        })
    }
}

impl From<Side> for &'static str {
    fn from(value: Side) -> Self {
        match value {
            Side::C2S => "C2S",
            Side::S2C => "S2C",
        }
    }
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
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => State::Handshake,
            1 => State::Login,
            2 => State::Play,
            3 => State::Status,
            _ => return Err("Bad state id"),
        })
    }
}

impl TryFrom<&str> for State {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Handshake" => State::Handshake,
            "Login" => State::Login,
            "Play" => State::Play,
            "Status" => State::Status,
            _ => return Err("Bad state"),
        })
    }
}

impl TryFrom<State> for u8 {
    type Error = ();

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
#[derive(Lua, Clone, Debug)]
pub enum Packet<'a> {
    C2S(C2SPacket<'a>),
    S2C(S2CPacket<'a>),
}
