//! Clientbound handshake packets.
//!
//! Stub: the protocol has no clientbound handshake packets, so this enum stays
//! empty. It exists so the dispatch code has a uniform shape per state.
//!
//! `EncodePacket` is deliberately not derived: an empty enum has no variants,
//! so the generated `match self {}` would not compile. There is nothing to
//! encode anyway.

use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Handshake {}
