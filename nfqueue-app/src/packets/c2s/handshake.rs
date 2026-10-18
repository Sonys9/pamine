use pcore::{Decode, decoder::VarInt};
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Handshake<'a> {
    #[id(0x00)]
    Handshaking(Handshaking<'a>)
}

#[derive(Decode, Debug, Clone)]
pub struct Handshaking<'a> {
    protocol: VarInt,
    address: &'a [u8],
    port: u16,
    next_state: VarInt,
}
