//! Clientbound login packets (protocol 760 / 1.19.2).

use pcore::{
    Decode, Encode,
    decoder::{Decoder, MAX_ELEMENTS, Uuid, VarInt},
    encoder::Encoder,
    errors::Error,
};
use pcore_gen::{DecodePacket, EncodePacket};

#[derive(DecodePacket, EncodePacket, Clone, Debug)]
pub enum Login<'a> {
    #[id(0x00)]
    Disconnect(Disconnect<'a>),
    #[id(0x01)]
    EncryptionRequest(EncryptionRequest<'a>),
    #[id(0x02)]
    LoginSuccess(LoginSuccess<'a>),
    #[id(0x03)]
    SetCompression(SetCompression),
    #[id(0x04)]
    LoginPluginRequest(LoginPluginRequest<'a>),
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct Disconnect<'a> {
    pub reason: &'a str,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct EncryptionRequest<'a> {
    pub server_id: &'a str,
    pub public_key: &'a [u8],
    pub verify_token: &'a [u8],
}

/// One signed skin/cape property of a player profile.
///
/// `String (32767)` in the wiki is a protocol String — VarInt length plus a
/// maximum character count — not the NBT `u16` framing.
#[derive(Debug, Clone)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub signature: Option<&'a str>,
}

impl<'a> Decode<'a> for Property<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let name = decoder.string()?;
        let value = decoder.string()?;
        // `Is Signed` gates the signature.
        let signature = if decoder.bool()? {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self {
            name,
            value,
            signature,
        })
    }
}

impl Encode for Property<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.name);
        encoder.string(self.value);
        match self.signature {
            Some(signature) => {
                encoder.bool(true);
                encoder.string(signature);
            }
            None => encoder.bool(false),
        }
    }
}

/// The `Property` array of `Login Success`.
///
/// Kept as its own type so the count and the elements are read in one place,
/// with the same `MAX_ELEMENTS` guard the other counted arrays use.
#[derive(Debug, Clone)]
pub struct Properties<'a> {
    pub entries: Vec<Property<'a>>,
}

impl<'a> Decode<'a> for Properties<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let len = decoder.varint()?.get();
        if len < 0 {
            return Err(Error::Read(0));
        }
        let len = len as usize;
        // Reject a hostile count before allocating: the length arrives from the
        // wire, so it can claim millions of entries that are not there.
        if len > MAX_ELEMENTS {
            return Err(Error::LenLimit(len));
        }
        let mut entries = Vec::with_capacity(len);
        for _ in 0..len {
            entries.push(Property::decode(decoder)?);
        }
        Ok(Self { entries })
    }
}

impl Encode for Properties<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.varint(self.entries.len() as i32);
        for entry in &self.entries {
            entry.encode(encoder);
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoginSuccess<'a> {
    pub uuid: Uuid,
    pub username: &'a str,
    pub properties: Properties<'a>,
}

impl<'a> Decode<'a> for LoginSuccess<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let uuid = decoder.uuid()?;
        let username = decoder.string()?;
        let properties = Properties::decode(decoder)?;
        Ok(Self {
            uuid,
            username,
            properties,
        })
    }
}

impl Encode for LoginSuccess<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.uuid(&self.uuid);
        encoder.string(self.username);
        self.properties.encode(encoder);
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetCompression {
    pub threshold: VarInt,
}

/// `Login Plugin Request` (0x04).
///
/// The payload has no length prefix: the wiki states its length "must be
/// inferred from the packet length", so it is read as the rest of the buffer.
#[derive(Decode, Encode, Debug, Clone)]
pub struct LoginPluginRequest<'a> {
    pub message_id: VarInt,
    pub channel: &'a str,
    #[decode(rest)]
    pub data: &'a [u8],
}
