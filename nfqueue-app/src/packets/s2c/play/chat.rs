//! Chat, signed messages and tab-list text.

use pcore::{
    Decode,
    decoder::{Decoder, Uuid, VarInt},
    errors::Error,
};

use crate::packets::{counted, optional_bytes, optional_chat};

/// `Chat Preview` (0x0C).
///
/// `Component is Present` is the gate for the message, so the message is read
/// by hand instead of through `Option<&str>` (which would consume a second
/// boolean).
#[derive(Debug, Clone)]
pub struct ChatPreview<'a> {
    pub query_id: i32,
    pub has_component: bool,
    pub message: Option<&'a str>,
}

impl<'a> Decode<'a> for ChatPreview<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let query_id = decoder.i32()?;
        let has_component = decoder.bool()?;
        let message = if has_component {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self {
            query_id,
            has_component,
            message,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct HideMessage<'a> {
    pub signature: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct Disconnect<'a> {
    pub reason: &'a str,
}

/// `Message Header` (0x32).
#[derive(Debug, Clone)]
pub struct MessageHeader<'a> {
    pub has_preceding_signature: bool,
    pub preceding_signature: Option<&'a [u8]>,
    pub sender: Uuid,
}

impl<'a> Decode<'a> for MessageHeader<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let has_preceding_signature = decoder.bool()?;
        let preceding_signature = if has_preceding_signature {
            Some(decoder.slice()?)
        } else {
            None
        };
        let sender = decoder.uuid()?;
        Ok(Self {
            has_preceding_signature,
            preceding_signature,
            sender,
        })
    }
}

/// A previously sent message, referenced by its sender and signature.
#[derive(Debug, Clone)]
pub struct PreviousMessage<'a> {
    pub sender: Uuid,
    pub signature: &'a [u8],
}

/// `Player Chat Message` (0x33).
///
/// The wire layout is a chain of gated sections, so it is decoded by hand:
///
/// ```text
/// signature present: bool
///   signature: Byte Array
/// sender: UUID
/// header signature: Byte Array
/// plain message: String
/// formatted present: bool
///   formatted message: Chat
/// timestamp: Long
/// salt: Long
/// previous messages: VarInt count of (UUID, Byte Array)
/// unsigned content present: bool
///   unsigned content: Chat
/// filter type: VarInt
///   filter mask: BitSet (only when filter type == 2)
/// network target: VarInt
/// network name: Chat
/// network target name present: bool
///   network target name: Chat
/// ```
#[derive(Debug, Clone)]
pub struct PlayerChatMessage<'a> {
    pub signature: Option<&'a [u8]>,
    pub sender: Uuid,
    pub header_signature: &'a [u8],
    pub plain_message: &'a str,
    pub formatted_message: Option<&'a str>,
    pub timestamp: i64,
    pub salt: i64,
    pub previous_messages: Vec<PreviousMessage<'a>>,
    pub unsigned_content: Option<&'a str>,
    pub filter_type: VarInt,
    /// Only present when `filter_type == 2` (partially filtered).
    pub filter_mask: Option<&'a [u8]>,
    pub network_target: VarInt,
    pub network_name: &'a str,
    pub network_target_name: Option<&'a str>,
}

/// `PASS_THROUGH` — no filters applied.
pub const CHAT_FILTER_PASS_THROUGH: i32 = 0;
/// `FULLY_FILTERED` — all filters applied.
pub const CHAT_FILTER_FULLY_FILTERED: i32 = 1;
/// `PARTIALLY_FILTERED` — only some filters applied; a mask follows.
pub const CHAT_FILTER_PARTIALLY_FILTERED: i32 = 2;

impl<'a> Decode<'a> for PlayerChatMessage<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let signature = optional_bytes(decoder)?;
        let sender = decoder.uuid()?;
        let header_signature = decoder.slice()?;
        let plain_message = decoder.string()?;
        let formatted_message = optional_chat(decoder)?;
        let timestamp = decoder.i64()?;
        let salt = decoder.i64()?;
        let previous_messages = counted(decoder, |d| {
            Ok(PreviousMessage {
                sender: d.uuid()?,
                signature: d.slice()?,
            })
        })?;
        let unsigned_content = optional_chat(decoder)?;
        let filter_type = decoder.varint()?;
        // The mask is only on the wire for the "partially filtered" type.
        let filter_mask = if filter_type.get() == CHAT_FILTER_PARTIALLY_FILTERED {
            Some(bitset(decoder)?)
        } else {
            None
        };
        let network_target = decoder.varint()?;
        let network_name = decoder.string()?;
        let network_target_name = optional_chat(decoder)?;
        Ok(Self {
            signature,
            sender,
            header_signature,
            plain_message,
            formatted_message,
            timestamp,
            salt,
            previous_messages,
            unsigned_content,
            filter_type,
            filter_mask,
            network_target,
            network_name,
            network_target_name,
        })
    }
}

/// A BitSet is a VarInt count of `i64` words, then that many words.
fn bitset<'a>(decoder: &mut Decoder<'a>) -> Result<&'a [u8], Error> {
    let count = decoder.varint()?.get();
    if count < 0 {
        return Err(Error::Nbt("negative bitset length"));
    }
    decoder.read_len(count as usize * 8)
}

#[derive(Decode, Debug, Clone)]
pub struct SystemChatMessage<'a> {
    pub json_data: &'a str,
    pub overlay: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetTabListHeaderAndFooter<'a> {
    pub header: &'a str,
    pub footer: &'a str,
}

#[derive(Debug, Clone)]
pub struct ChatSuggestions<'a> {
    pub action: VarInt,
    pub entries: Vec<&'a str>,
}

impl<'a> Decode<'a> for ChatSuggestions<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let action = decoder.varint()?;
        let entries = counted(decoder, |d| d.string())?;
        Ok(Self { action, entries })
    }
}
