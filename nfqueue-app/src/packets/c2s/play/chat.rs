//! Chat messages, signed commands and message acknowledgements.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, Uuid},
    encoder::Encoder,
    errors::Error,
};

use crate::packets::{counted, optional_bytes};

/// `Message Acknowledgment` (0x03).
///
/// Hand-written `Decode`: the profile id and signature are gated by `has_data`.
#[derive(Debug, Clone)]
pub struct MessageAcknowledgment<'a> {
    pub has_data: bool,
    pub profile_id: Option<Uuid>,
    /// `None` when the length prefix is `0`.
    pub last_signature: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for MessageAcknowledgment<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let has_data = decoder.bool()?;
        let (profile_id, last_signature) = if has_data {
            (Some(decoder.uuid()?), optional_bytes(decoder)?)
        } else {
            (None, None)
        };
        Ok(Self {
            has_data,
            profile_id,
            last_signature,
        })
    }
}

impl Encode for MessageAcknowledgment<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.bool(self.has_data);
        if self.has_data {
            encoder.uuid(&self.profile_id.unwrap_or(Uuid::new([0; 16])));
            encoder.bytes(self.last_signature.unwrap_or(&[]));
        }
    }
}

/// One `(argument name, signature)` pair of a signed chat command.
#[derive(Debug, Clone)]
pub struct ArgumentSignature<'a> {
    pub name: &'a str,
    pub signature: &'a [u8],
}

impl Encode for ArgumentSignature<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.name);
        encoder.bytes(self.signature);
    }
}

/// `Chat Command` (0x04).
///
/// In 1.19.2 the packet ends after `signed_preview`; the "last seen" and
/// "last received" arrays only exist on `Chat Message`.
#[derive(Debug, Clone)]
pub struct ChatCommand<'a> {
    pub command: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignature<'a>>,
    pub signed_preview: bool,
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
        Ok(Self {
            command,
            timestamp,
            salt,
            argument_signatures,
            signed_preview,
        })
    }
}

impl Encode for ChatCommand<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.command);
        encoder.i64(self.timestamp);
        encoder.i64(self.salt);
        encoder.encode(&self.argument_signatures);
        encoder.bool(self.signed_preview);
    }
}

/// A previously seen chat message: its sender and signature.
#[derive(Debug, Clone)]
pub struct LastSeenMessage<'a> {
    pub sender: Uuid,
    pub signature: &'a [u8],
}

impl Encode for LastSeenMessage<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.uuid(&self.sender);
        encoder.bytes(self.signature);
    }
}

/// `Chat Message` (0x05).
///
/// Hand-written `Decode`: the signature is length-prefixed (empty means
/// absent), and the "last received" entry is gated by a boolean.
#[derive(Debug, Clone)]
pub struct ChatMessage<'a> {
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    /// `None` when the signature length prefix is `0`.
    pub signature: Option<&'a [u8]>,
    pub signed_preview: bool,
    pub last_seen_messages: Vec<LastSeenMessage<'a>>,
    pub last_received: Option<LastSeenMessage<'a>>,
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
                sender: d.uuid()?,
                signature: d.slice()?,
            })
        })?;
        let last_received = if decoder.bool()? {
            Some(LastSeenMessage {
                sender: decoder.uuid()?,
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

impl Encode for ChatMessage<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.message);
        encoder.i64(self.timestamp);
        encoder.i64(self.salt);
        // A zero-length signature means "not signed".
        encoder.bytes(self.signature.unwrap_or(&[]));
        encoder.bool(self.signed_preview);
        encoder.encode(&self.last_seen_messages);
        match &self.last_received {
            Some(last) => {
                encoder.bool(true);
                last.encode(encoder);
            }
            None => encoder.bool(false),
        }
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct ChatPreview<'a> {
    pub query: i32,
    pub message: &'a str,
}
