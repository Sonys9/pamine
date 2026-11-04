//! Packet inspection and rewriting.
//!
//! [`on_packet`] is the single hook the relay calls for every parsed packet. It
//! returns an [`Update`] describing what to do:
//!
//! - `change_with: Some(bytes)` replaces the packet with `bytes`, which must
//!   already be *framed* — one `len:VarInt [data_len:VarInt] body` per packet,
//!   concatenated. `Some(Vec::new())` drops the packet.
//! - `change_with: None` forwards the original packet untouched.
//! - `inject: Some(injection)` queues an extra packet for the opposite
//!   direction.
//!
//! Concatenating frames is safe: TCP is a byte stream, so the peer reads them
//! back one at a time. That is why the hook takes the threshold and does its
//! own framing — it lets several packets go out in a single `write_all`
//! instead of one syscall each.

use std::time::{SystemTime, UNIX_EPOCH};

use pcore::{decoder::VarInt, encode};
use tracing::{error, info};

use crate::{
    connections::{Injection, Update},
    ids,
    packets::{
        Packet::{self, C2S},
        UnsureThreshold,
        c2s::{
            self,
            packets::C2SPacket,
            play::{ChatCommand, ChatMessage, SetCreativeModeSlot},
        },
        frame_packet,
        s2c::{self, play::SynchronizePlayerPosition},
    },
};

/// Decides whether a packet should be rewritten.
///
/// Errors are logged and turned into [`Update::none`]: a broken rewrite must not
/// take the connection down.
pub fn on_packet(packet: &Packet<'_>, threshold: &UnsureThreshold) -> Update {
    let result = match packet {
        C2S(C2SPacket::Play(c2s::play::Play::ChatMessage(message))) => {
            on_message(message, threshold)
        }
        C2S(C2SPacket::Play(c2s::play::Play::SetCreativeModeSlot(packet))) => {
            on_creative_slot(packet)
        }
        _ => return Update::none(),
    };

    result.unwrap_or_else(|e| {
        error!("on_packet failed: {e}");
        Update::none()
    })
}

fn on_creative_slot(packet: &SetCreativeModeSlot) -> eyre::Result<Update> {
    info!("Packet: {:?}", packet);
    info!("Item: {:?}", packet.clicked_item);
    if let Some(item_id) = packet.clicked_item.item_id {
        info!("Item: {:?}", ids::Item::by_id(item_id.get() as u16));
    };
    Ok(Update::none())
}

/// Rewrites a chat message.
///
/// Each body is wrapped back into the enum and encoded as a whole, so the opcode
/// comes from `#[id(0x05)]` on `Play::ChatMessage` and is never spelled out by
/// hand.
fn on_message(packet: &ChatMessage<'_>, threshold: &UnsureThreshold) -> eyre::Result<Update> {
    info!("The local player just typed '{}'", packet.message);

    match packet.message {
        "info" => spoof_info(packet, threshold),
        "change my position" => change_pos(threshold),
        _ => Ok(Update::none()),
    }
}

/// Teleports the client by injecting a clientbound `Synchronize Player Position`.
///
/// The packet is S2C, so it is queued for the opposite direction: the C2S relay
/// sees the chat message and hands the injection to the S2C relay, which writes
/// it to the client. `changeable: false` keeps the injection out of the rewrite
/// hook, so it cannot feed back into itself.
fn change_pos(threshold: &UnsureThreshold) -> eyre::Result<Update> {
    let position = SynchronizePlayerPosition {
        x: 100.,
        y: 100.,
        z: 100.,
        yaw: 100.,
        pitch: 100.,
        flags: 0,
        teleport_id: VarInt::new(123456),
        dismount_vehicle: true,
    };
    let body = encode(&s2c::play::Play::SynchronizePlayerPosition(position));
    let framed = frame_packet(&body, threshold)?;

    Ok(Update {
        change_with: None,
        inject: Some(Injection::new(framed, false)),
    })
}

/// Replaces the message with two spoofed ones plus a command.
fn spoof_info(packet: &ChatMessage<'_>, threshold: &UnsureThreshold) -> eyre::Result<Update> {
    let messages = ["This message was just spoofed!", "Using a command '/?'."];
    let mut result = Vec::new();
    for message in messages {
        let mut packet = packet.clone();
        packet.message = message;
        let message_packet = encode(&c2s::play::Play::ChatMessage(packet));
        result.extend_from_slice(&frame_packet(&message_packet, threshold)?);
    }

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
    result.extend_from_slice(&frame_packet(&command_packet, threshold)?);

    Ok(Update {
        change_with: Some(result),
        inject: None,
    })
}
