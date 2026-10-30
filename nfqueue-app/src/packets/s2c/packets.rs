//! Union of all clientbound packets, tagged by protocol phase.

use crate::packets::s2c::{handshake::Handshake, login::Login, play::Play, status::Status};

#[derive(Clone, Debug)]
pub enum S2CPacket {
    Handshake(Handshake),
    Login(Login),
    Play(Play),
    Status(Status),
}
