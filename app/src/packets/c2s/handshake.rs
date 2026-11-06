//! Serverbound handshake packets (protocol 760 / 1.19.2).

use pcore::{Decode, decoder::VarInt};
use pcore_gen::{DecodePacket, Encode, EncodePacket, Lua, LuaPacket};

#[derive(DecodePacket, EncodePacket, Lua, LuaPacket, Clone, Debug)]
pub enum Handshake<'a> {
    #[id(0x00)]
    Handshaking(Handshaking<'a>),
}

#[derive(Decode, Encode, Lua, LuaPacket, Debug, Clone)]
pub struct Handshaking<'a> {
    pub protocol: VarInt,
    pub address: &'a str,
    pub port: u16,
    pub next_state: VarInt,
}
