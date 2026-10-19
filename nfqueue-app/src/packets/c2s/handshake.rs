use pcore::{Decode, decoder::VarInt};
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Handshake<'a> {
    #[id(0x00)]
    Handshaking(Handshaking<'a>),
}

#[derive(Decode, Debug, Clone)]
pub struct Handshaking<'a> {
    pub protocol: VarInt,
    pub address: &'a [u8],
    pub port: u16,
    pub next_state: VarInt,
}