//! Clientbound handshake packets.
//!
//! Stub: the protocol has no clientbound handshake packets, so this enum stays
//! empty. It exists so the dispatch code has a uniform shape per state.
//!
//! `EncodePacket` is deliberately not derived: an empty enum has no variants,
//! so the generated `match self {}` would not compile. There is nothing to
//! encode anyway.

use pcore_gen::{DecodePacket, Lua, LuaPacket};

#[derive(DecodePacket, Lua, LuaPacket, Clone, Debug)]
pub enum Handshake {}

impl Handshake {
    /// The wire opcode of this packet.
    ///
    /// The protocol has no clientbound handshake packets, so this enum is
    /// uninhabited and the method can never actually be called. It exists so
    /// [`S2CPacket::opcode`](crate::packets::s2c::packets::S2CPacket::opcode)
    /// can delegate to every phase uniformly.
    pub fn opcode(&self) -> i32 {
        match *self {}
    }
}
