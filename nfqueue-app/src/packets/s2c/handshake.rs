//! Clientbound handshake packets.
//!
//! Stub: the protocol has no clientbound handshake packets, so this enum stays
//! empty. It exists so the dispatch code has a uniform shape per state.

use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Handshake {}
