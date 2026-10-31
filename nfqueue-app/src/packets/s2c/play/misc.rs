//! Packets that do not fit a larger subject group.

use pcore::{
    Decode, Encode,
    decoder::{Decoder, NbtTag, VarInt},
    encoder::Encoder,
    errors::Error,
};

#[derive(Decode, Encode, Debug, Clone)]
pub struct PluginMessage<'a> {
    pub channel: &'a str,
    /// Everything after the channel: the payload has no length prefix, its
    /// length is whatever remains in the packet. This is genuinely opaque —
    /// the layout is defined by the plugin, not by the protocol.
    #[decode(rest)]
    pub data: &'a [u8],
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct Ping {
    pub id: i32,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct KeepAlive {
    pub keep_alive_id: i64,
}

/// `Server Data` (0x42).
///
/// Hand-written `Decode`: `motd` and `icon` are gated by their own booleans,
/// which `Option<T>` would consume a second time.
#[derive(Debug, Clone)]
pub struct ServerData<'a> {
    pub has_motd: bool,
    pub motd: Option<&'a str>,
    pub has_icon: bool,
    /// Base64-encoded PNG.
    pub icon: Option<&'a str>,
    pub previews_chat: bool,
    pub enforces_secure_chat: bool,
}

impl<'a> Decode<'a> for ServerData<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let has_motd = decoder.bool()?;
        let motd = if has_motd {
            Some(decoder.string()?)
        } else {
            None
        };
        let has_icon = decoder.bool()?;
        let icon = if has_icon {
            Some(decoder.string()?)
        } else {
            None
        };
        let previews_chat = decoder.bool()?;
        let enforces_secure_chat = decoder.bool()?;
        Ok(Self {
            has_motd,
            motd,
            has_icon,
            icon,
            previews_chat,
            enforces_secure_chat,
        })
    }
}

impl Encode for ServerData<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.bool(self.has_motd);
        if self.has_motd {
            encoder.string(self.motd.unwrap_or(""));
        }
        encoder.bool(self.has_icon);
        if self.has_icon {
            encoder.string(self.icon.unwrap_or(""));
        }
        encoder.bool(self.previews_chat);
        encoder.bool(self.enforces_secure_chat);
    }
}

/// `Resource Pack` (0x3D).
///
/// Hand-written `Decode`: `prompt_message` is gated by `has_prompt_message`.
#[derive(Debug, Clone)]
pub struct ResourcePack<'a> {
    pub url: &'a str,
    /// 40-character lowercase hex SHA-1, or an arbitrary unique id.
    pub hash: &'a str,
    pub forced: bool,
    pub has_prompt_message: bool,
    pub prompt_message: Option<&'a str>,
}

impl<'a> Decode<'a> for ResourcePack<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let url = decoder.string()?;
        let hash = decoder.string()?;
        let forced = decoder.bool()?;
        let has_prompt_message = decoder.bool()?;
        let prompt_message = if has_prompt_message {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self {
            url,
            hash,
            forced,
            has_prompt_message,
            prompt_message,
        })
    }
}

impl Encode for ResourcePack<'_> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.string(self.url);
        encoder.string(self.hash);
        encoder.bool(self.forced);
        encoder.bool(self.has_prompt_message);
        if self.has_prompt_message {
            encoder.string(self.prompt_message.unwrap_or(""));
        }
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct TagQueryResponse<'a> {
    pub transaction_id: VarInt,
    pub nbt: NbtTag<'a>,
}
