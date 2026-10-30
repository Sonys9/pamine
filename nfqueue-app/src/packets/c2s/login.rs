//! Serverbound login packets (protocol 760 / 1.19.2).

use pcore::{
    Decode,
    decoder::{Decoder, Uuid, VarInt},
    errors::Error,
};
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Login<'a> {
    #[id(0x00)]
    LoginStart(LoginStart<'a>),
    #[id(0x01)]
    EncryptionResponse(EncryptionResponse<'a>),
    #[id(0x02)]
    LoginPluginResponse(LoginPluginResponse<'a>),
}

#[derive(Decode, Debug, Clone)]
pub struct LoginStart<'a> {
    pub name: &'a str,
    pub sig_data: Option<SigData<'a>>,
    pub player_uuid: Option<Uuid>,
}

#[derive(Decode, Debug, Clone)]
pub struct SigData<'a> {
    pub timestamp: i64,
    pub public_key: &'a [u8],
    pub signature: &'a [u8],
}

/// `Encryption Response` (0x01).
///
/// Hand-written `Decode`: `has_verify_token` selects between two mutually
/// exclusive tails, so neither can be an `Option` (which would read its own
/// boolean).
#[derive(Debug, Clone)]
pub struct EncryptionResponse<'a> {
    pub shared_secret: &'a [u8],
    pub has_verify_token: bool,
    /// Present when `has_verify_token` is set.
    pub verify_token: Option<&'a [u8]>,
    /// Present when `has_verify_token` is clear.
    pub salt: Option<i64>,
    /// Present when `has_verify_token` is clear.
    pub message_signature: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for EncryptionResponse<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let shared_secret = decoder.slice()?;
        let has_verify_token = decoder.bool()?;
        let (verify_token, salt, message_signature) = if has_verify_token {
            (Some(decoder.slice()?), None, None)
        } else {
            (None, Some(decoder.i64()?), Some(decoder.slice()?))
        };
        Ok(Self {
            shared_secret,
            has_verify_token,
            verify_token,
            salt,
            message_signature,
        })
    }
}

/// `Login Plugin Response` (0x02).
///
/// Hand-written `Decode`: `data` is an *Optional Byte Array* — a VarInt length
/// where `0` means absent — not a boolean-gated field.
#[derive(Debug, Clone)]
pub struct LoginPluginResponse<'a> {
    pub message_id: VarInt,
    pub successful: bool,
    pub data: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for LoginPluginResponse<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let message_id = decoder.varint()?;
        let successful = decoder.bool()?;
        // A zero-length array is the "no payload" case.
        let data = if successful {
            let bytes = decoder.slice()?;
            (!bytes.is_empty()).then_some(bytes)
        } else {
            None
        };
        Ok(Self {
            message_id,
            successful,
            data,
        })
    }
}
