use crate::packets::s2c::{handshake::Handshake, login::Login, play::Play, status::Status};

#[derive(Clone, Debug)]
pub enum S2CPacket<'a> {
    Handshake(Handshake),
    Login(Login<'a>),
    Play(Play<'a>),
    Status(Status<'a>),
}
