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
