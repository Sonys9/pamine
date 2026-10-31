//! Packet inspection and rewriting.
//!
//! [`on_packet`] is the single hook the relay calls for every parsed packet.
//! Returning `None` forwards the packet untouched; returning `Some(body)`
//! replaces it with `body`, which must be a packet *body* (opcode + fields) —
//! the relay adds the frame around it, because only it knows the current
//! compression threshold.

use pcore::encode;
use tracing::info;

use crate::packets::{
    Packet::{self, C2S},
    c2s::{self, packets::C2SPacket, play::ChatMessage},
};

/// Decides whether a packet should be rewritten.
///
/// Returns the replacement body, or `None` to forward the original packet.
pub fn on_packet(packet: &Packet<'_>) -> Option<Vec<u8>> {
    match packet {
        C2S(C2SPacket::Play(c2s::play::Play::ChatMessage(message))) => on_message(message),
        _ => None,
    }
}

/// Rewrites a chat message.
///
/// The body is wrapped back into the enum and encoded as a whole, so the opcode
/// comes from `#[id(0x05)]` on `Play::ChatMessage` and is never spelled out by
/// hand.
fn on_message(message: &ChatMessage<'_>) -> Option<Vec<u8>> {
    info!("The local player just typed '{}'", message.message);
    if message.message != "Hello, world!" {
        return None;
    };

    info!("Spoofing with the custom message");
    info!("Before: {:?}", message);
    let mut message = message.clone();
    message.message = "!dlrow ,olleH";
    info!("After: {:?}", message);

    let packet = c2s::play::Play::ChatMessage(message);
    Some(encode(&packet))
}
