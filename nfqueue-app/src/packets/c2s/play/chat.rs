//! Chat messages, signed commands and message acknowledgements.

use pcore::{
    Decode,
    decoder::{Decoder, Uuid, VarInt},
    errors::Error,
};

use crate::packets::{counted, optional_bytes};

#[derive(Debug, Clone)]
pub struct MessageAcknowledgment<'a> {
    pub message_count: VarInt,
    pub has_data: bool,
    /// Only present when `has_data` is set.
    pub profile_id: Option<Uuid>,
    /// Only present when `has_data` is set; `None` when the length is `0`.
    pub last_signature: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for MessageAcknowledgment<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let message_count = decoder.varint()?;
        let has_data = decoder.bool()?;
        let (profile_id, last_signature) = if has_data {
            (Some(decoder.uuid()?), optional_bytes(decoder)?)
        } else {
            (None, None)
        };
        Ok(Self {
            message_count,
            has_data,
            profile_id,
            last_signature,
        })
    }
}

/// One `(argument name, signature)` pair of a signed chat command.
#[derive(Debug, Clone)]
pub struct ArgumentSignature<'a> {
    pub name: &'a str,
    pub signature: &'a [u8],
}

/// A previously seen chat message, referenced by its signature and id.
#[derive(Debug, Clone)]
pub struct LastSeenMessage<'a> {
    pub signature: &'a [u8],
    pub message_id: VarInt,
}

/// The most recently received chat message.
#[derive(Debug, Clone)]
pub struct LastReceived<'a> {
    pub message_id: VarInt,
    pub signature: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct ChatCommand<'a> {
    pub command: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignature<'a>>,
    pub signed_preview: bool,
    pub last_seen_messages: Vec<LastSeenMessage<'a>>,
    pub last_received: Option<LastReceived<'a>>,
}

impl<'a> Decode<'a> for ChatCommand<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let command = decoder.string()?;
        let timestamp = decoder.i64()?;
        let salt = decoder.i64()?;
        let argument_signatures = counted(decoder, |d| {
            Ok(ArgumentSignature {
                name: d.string()?,
                signature: d.slice()?,
            })
        })?;
        let signed_preview = decoder.bool()?;
        let last_seen_messages = counted(decoder, |d| {
            Ok(LastSeenMessage {
                signature: d.slice()?,
                message_id: d.varint()?,
            })
        })?;
        let last_received = if decoder.bool()? {
            Some(LastReceived {
                message_id: decoder.varint()?,
                signature: decoder.slice()?,
            })
        } else {
            None
        };
        Ok(Self {
            command,
            timestamp,
            salt,
            argument_signatures,
            signed_preview,
            last_seen_messages,
            last_received,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage<'a> {
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    /// `None` when the signature length prefix is `0`.
    pub signature: Option<&'a [u8]>,
    pub signed_preview: bool,
    pub last_seen_messages: Vec<LastSeenMessage<'a>>,
    pub last_received: Option<LastReceived<'a>>,
}

impl<'a> Decode<'a> for ChatMessage<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let message = decoder.string()?;
        let timestamp = decoder.i64()?;
        let salt = decoder.i64()?;
        let signature = optional_bytes(decoder)?;
        let signed_preview = decoder.bool()?;
        let last_seen_messages = counted(decoder, |d| {
            Ok(LastSeenMessage {
                signature: d.slice()?,
                message_id: d.varint()?,
            })
        })?;
        let last_received = if decoder.bool()? {
            Some(LastReceived {
                message_id: decoder.varint()?,
                signature: decoder.slice()?,
            })
        } else {
            None
        };
        Ok(Self {
            message,
            timestamp,
            salt,
            signature,
            signed_preview,
            last_seen_messages,
            last_received,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct ChatPreview<'a> {
    pub query: i32,
    pub message: &'a str,
}
