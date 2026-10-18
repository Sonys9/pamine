use crate::packets::c2s::handshake::Handshake;

#[derive(Clone, Debug)]
pub enum C2SPacket<'a> {
    Handshake(Handshake<'a>)
}