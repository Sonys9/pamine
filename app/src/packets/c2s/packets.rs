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
