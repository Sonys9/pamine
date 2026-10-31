//! Per-state packet dispatch.
//!
//! Each function decodes one packet body for the given protocol phase and
//! returns it wrapped in [`Packet`]. State transitions happen here, because
//! they are driven by the packet contents (`Handshake.next_state`,
//! `Login Success`, `Set Compression`).

use pcore::decoder::Decoder;

use crate::packets::{
    c2s::{self, packets::C2SPacket},
    s2c::{self, packets::S2CPacket},
};

use super::state::{Packet, Side, State};

pub(super) fn handshake<'a>(
    state: &mut State,
    opcode: i32,
    mut decoder: Decoder<'a>,
    side: Side,
) -> eyre::Result<Packet<'a>> {
    let Side::C2S = side else {
        eyre::bail!("server sent a handshake packet");
    };

    let parsed = c2s::handshake::Handshake::decode_by_opcode(opcode, &mut decoder)?;
    let c2s::handshake::Handshake::Handshaking(ref packet) = parsed;
    *state = match packet.next_state.get() {
        1 => State::Status,
        _ => State::Login,
    };
    Ok(Packet::C2S(C2SPacket::Handshake(parsed)))
}

pub(super) fn login<'a>(
    state: &mut State,
    opcode: i32,
    mut decoder: Decoder<'a>,
    side: Side,
    threshold: &mut Option<i32>,
) -> eyre::Result<Packet<'a>> {
    match side {
        Side::C2S => {
            let parsed = c2s::login::Login::decode_by_opcode(opcode, &mut decoder)?;
            Ok(Packet::C2S(C2SPacket::Login(parsed)))
        }
        Side::S2C => {
            let parsed = s2c::login::Login::decode_by_opcode(opcode, &mut decoder)?;
            match &parsed {
                s2c::login::Login::LoginSuccess(_) => *state = State::Play,
                s2c::login::Login::SetCompression(set) => {
                    let value = set.threshold.get();
                    // A negative threshold disables compression entirely.
                    *threshold = (value >= 0).then_some(value);
                }
                _ => {}
            }
            Ok(Packet::S2C(S2CPacket::Login(parsed)))
        }
    }
}

pub(super) fn status<'a>(
    opcode: i32,
    mut decoder: Decoder<'a>,
    side: Side,
) -> eyre::Result<Packet<'a>> {
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

pub(super) fn play<'a>(
    opcode: i32,
    mut decoder: Decoder<'a>,
    side: Side,
) -> eyre::Result<Packet<'a>> {
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
