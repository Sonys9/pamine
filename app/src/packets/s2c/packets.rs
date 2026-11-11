//! Union of all clientbound packets, tagged by protocol phase.

use pcore_gen::Lua;

use crate::packets::s2c::{handshake::Handshake, login::Login, play::Play, status::Status};

#[derive(Lua, Clone, Debug)]
pub enum S2CPacket<'a> {
    Handshake(Handshake),
    Login(Login<'a>),
    Play(Play<'a>),
    Status(Status<'a>),
}

impl S2CPacket<'_> {
    /// The wire opcode of the wrapped packet, taken from its `#[id(N)]`.
    ///
    /// Each phase enum already exposes `opcode()`; this forwards to it so a
    /// caller holding an [`S2CPacket`] does not have to match the phase first.
    pub fn opcode(&self) -> i32 {
        match self {
            Self::Handshake(packet) => packet.opcode(),
            Self::Login(packet) => packet.opcode(),
            Self::Play(packet) => packet.opcode(),
            Self::Status(packet) => packet.opcode(),
        }
    }
}
