//! Packet inspection and rewriting.
//!
//! [`on_packet`] is the single hook the relay calls for every parsed packet.
//! Returning `None` forwards the packet untouched; returning `Some(bytes)`
//! replaces it with `bytes`, which must already be *framed* — one
//! `len:VarInt [data_len:VarInt] body` per packet, concatenated.
//!
//! Concatenating frames is safe: TCP is a byte stream, so the peer reads them
//! back one at a time. That is why the hook takes the threshold and does its
//! own framing — it lets several packets go out in a single `write_all`
//! instead of one syscall each.

use pcore::encode;
use tracing::info;

use crate::packets::{
    Packet::{self, C2S},
    c2s::{
        self,
        packets::C2SPacket,
        play::{ChatMessage, SetPlayerPosition},
    },
    frame_packet,
};

/// Decides whether a packet should be rewritten.
///
/// Returns the replacement frames, or `None` to forward the original packet.
pub fn on_packet(packet: &Packet<'_>, threshold: Option<i32>) -> Option<Vec<u8>> {
    match packet {
        C2S(C2SPacket::Play(c2s::play::Play::ChatMessage(message))) => {
            on_message(message, threshold)
        }
        _ => None,
    }
}

/// Rewrites a chat message.
///
/// Each body is wrapped back into the enum and encoded as a whole, so the opcode
/// comes from `#[id(0x05)]` on `Play::ChatMessage` and is never spelled out by
/// hand.
fn on_message(message: &ChatMessage<'_>, threshold: Option<i32>) -> Option<Vec<u8>> {
    info!("The local player just typed '{}'", message.message);
    if message.message != "Hello, world!" {
        return None;
    };

    info!("Spoofing with the custom message");
    info!("Before: {:?}", message);
    let mut message = message.clone();
    message.message = "!dlrow ,olleH";
    info!("After: {:?}", message);

    // Two packets, two frames, one buffer — so the relay needs a single write.
    let message_packet = encode(&c2s::play::Play::ChatMessage(message));
    let position_packet = encode(&c2s::play::Play::SetPlayerPosition(SetPlayerPosition {
        x: 60.,
        feet_y: 100.,
        z: -176.,
        on_ground: false,
    }));

    let mut out = frame_packet(&message_packet, threshold).ok()?;
    out.extend_from_slice(&frame_packet(&position_packet, threshold).ok()?);
    Some(out)
}
