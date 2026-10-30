//! Clientbound handshake packets.
//!
//! Stub: no packets implemented yet. The enum is empty, so every opcode falls
//! through to `Error::UnknownOpcode`.

use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Handshake {}
