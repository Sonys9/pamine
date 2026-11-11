//! Union of all serverbound packets, tagged by protocol phase.

use pcore_gen::Lua;

use crate::packets::c2s::{handshake::Handshake, login::Login, play::Play, status::Status};

#[derive(Lua, Clone, Debug)]
pub enum C2SPacket<'a> {
    Handshake(Handshake<'a>),
    Login(Login<'a>),
    Play(Play<'a>),
    Status(Status),
}

impl C2SPacket<'_> {
    /// The wire opcode of the wrapped packet, taken from its `#[id(N)]`.
    ///
    /// Each phase enum already exposes `opcode()`; this forwards to it so a
    /// caller holding a [`C2SPacket`] does not have to match the phase first.
    pub fn opcode(&self) -> i32 {
        match self {
            Self::Handshake(packet) => packet.opcode(),
            Self::Login(packet) => packet.opcode(),
            Self::Play(packet) => packet.opcode(),
            Self::Status(packet) => packet.opcode(),
        }
    }
}
