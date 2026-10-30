//! Union of all serverbound packets, tagged by protocol phase.

use crate::packets::c2s::{handshake::Handshake, login::Login, play::Play, status::Status};

#[derive(Clone, Debug)]
pub enum C2SPacket {
    Handshake(Handshake),
    Login(Login),
    Play(Play),
    Status(Status),
}
