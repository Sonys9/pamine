use crate::packets::s2c::handshake::Handshake;

#[derive(Clone, Debug)]
pub enum S2CPacket {
    Handshake(Handshake)
}