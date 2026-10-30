//! Per-state packet dispatch.
//!
//! Each function decodes one packet body for the given protocol phase and
//! returns it wrapped in [`Packet`]. State transitions happen here, because
//! they are driven by the packet contents.
//!
//! All packet enums are currently empty stubs, so every opcode falls through to
//! `Error::UnknownOpcode`. When packets are added, the state transitions below
//! need to be wired up again (see the comments).

use pcore::decoder::Decoder;

use crate::packets::{
    c2s::{self, packets::C2SPacket},
    s2c::{self, packets::S2CPacket},
};

use super::state::{Packet, Side, State};

pub(super) fn handshake(
    state: &mut State,
    opcode: i32,
    mut decoder: Decoder<'_>,
    side: Side,
) -> eyre::Result<Packet> {
    let Side::C2S = side else {
        eyre::bail!("server sent a handshake packet");
    };

    let parsed = c2s::handshake::Handshake::decode_by_opcode(opcode, &mut decoder)?;
    // TODO: once `Handshaking` exists, read `next_state` and set
    // `*state = State::Status` (1) or `State::Login` (otherwise).
    let _ = state;
    Ok(Packet::C2S(C2SPacket::Handshake(parsed)))
}

pub(super) fn login(
    state: &mut State,
    opcode: i32,
    mut decoder: Decoder<'_>,
    side: Side,
    threshold: &mut Option<i32>,
) -> eyre::Result<Packet> {
    match side {
        Side::C2S => {
            let parsed = c2s::login::Login::decode_by_opcode(opcode, &mut decoder)?;
            Ok(Packet::C2S(C2SPacket::Login(parsed)))
        }
        Side::S2C => {
            let parsed = s2c::login::Login::decode_by_opcode(opcode, &mut decoder)?;
            // TODO: once the variants exist, `LoginSuccess` moves to `State::Play`
            // and `SetCompression` updates `*threshold`.
            let _ = (state, threshold);
            Ok(Packet::S2C(S2CPacket::Login(parsed)))
        }
    }
}

pub(super) fn status(opcode: i32, mut decoder: Decoder<'_>, side: Side) -> eyre::Result<Packet> {
    match side {
        Side::C2S => {
            let parsed = c2s::status::Status::decode_by_opcode(opcode, &mut decoder)?;
            Ok(Packet::C2S(C2SPacket::Status(parsed)))
        }
        Side::S2C => {
            let parsed = s2c::status::Status::decode_by_opcode(opcode, &mut decoder)?;
            Ok(Packet::S2C(S2CPacket::Status(parsed)))
        }
    }
}

pub(super) fn play(opcode: i32, mut decoder: Decoder<'_>, side: Side) -> eyre::Result<Packet> {
    match side {
        Side::C2S => {
            let parsed = c2s::play::Play::decode_by_opcode(opcode, &mut decoder)?;
            Ok(Packet::C2S(C2SPacket::Play(parsed)))
        }
        Side::S2C => {
            let parsed = s2c::play::Play::decode_by_opcode(opcode, &mut decoder)?;
            Ok(Packet::S2C(S2CPacket::Play(parsed)))
        }
    }
}
