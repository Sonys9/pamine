//! Chat, signed messages and tab-list text.

use pcore::{
    Decode,
    decoder::{Decoder, Uuid, VarInt},
    errors::Error,
};

use crate::packets::{counted, optional_bytes, optional_chat};

#[derive(Decode, Debug, Clone)]
pub struct ChatPreview<'a> {
    pub query_id: i32,
    pub has_component: bool,
    pub message: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct HideMessage<'a> {
    pub signature: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct Disconnect<'a> {
    pub reason: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct MessageHeader<'a> {
    pub has_preceding_signature: bool,
    pub preceding_signature: Option<&'a [u8]>,
    pub sender: Uuid,
}

/// A previously sent message, referenced by its id and signature.
#[derive(Debug, Clone)]
pub struct PreviousMessage<'a> {
    pub message_id: VarInt,
    pub signature: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct PlayerChatMessage<'a> {
    pub sender: Uuid,
    pub index: VarInt,
    /// `None` when the signature length prefix is `0`.
    pub signature: Option<&'a [u8]>,
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub previous_messages: Vec<PreviousMessage<'a>>,
    pub unsigned_content: Option<&'a str>,
    pub filter_type: VarInt,
    pub network_target: VarInt,
    pub network_name: &'a str,
}

impl<'a> Decode<'a> for PlayerChatMessage<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let sender = decoder.uuid()?;
        let index = decoder.varint()?;
        let signature = optional_bytes(decoder)?;
        let message = decoder.string()?;
        let timestamp = decoder.i64()?;
        let salt = decoder.i64()?;
        let previous_messages = counted(decoder, |d| {
            Ok(PreviousMessage {
                message_id: d.varint()?,
                signature: d.slice()?,
            })
        })?;
        let unsigned_content = optional_chat(decoder)?;
        let filter_type = decoder.varint()?;
        let network_target = decoder.varint()?;
        let network_name = decoder.string()?;
        Ok(Self {
            sender,
            index,
            signature,
            message,
            timestamp,
            salt,
            previous_messages,
            unsigned_content,
            filter_type,
            network_target,
            network_name,
        })
    }
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
