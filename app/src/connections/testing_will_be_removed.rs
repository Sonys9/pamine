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

use std::time::{SystemTime, UNIX_EPOCH};

use pcore::encode;
use tracing::info;

use crate::packets::{
    Packet::{self, C2S}, c2s::{
        self, packets::C2SPacket, play::{ChatCommand, ChatMessage},
    }, frame_packet,
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
fn on_message(packet: &ChatMessage<'_>, threshold: Option<i32>) -> Option<Vec<u8>> {
    info!("The local player just typed '{}'", packet.message);
    if packet.message != "Info" {
        return None;
    };

    let messages = ["This message was just spoofed!", "Using a command '/?'."];
    let mut result = Vec::new();
    for message in messages {
        let mut packet = packet.clone();
        packet.message = message;
        let message_packet = encode(&c2s::play::Play::ChatMessage(packet));
        result.extend_from_slice(&frame_packet(&message_packet, threshold).ok()?);
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let command_packet = encode(&c2s::play::Play::ChatCommand(ChatCommand {
        command: "?",
        timestamp: now,
        salt: now,
        argument_signatures: Vec::new(),
        signed_preview: false,
        last_seen_messages: &[0, 0],
    }));
    result.extend_from_slice(&frame_packet(&command_packet, threshold).ok()?);

    Some(result)
}
