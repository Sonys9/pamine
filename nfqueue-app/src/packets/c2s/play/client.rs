//! Client settings, keep-alive and other connection-level packets.

use pcore::{
    Decode,
    decoder::{Decoder, VarInt},
    errors::Error,
};

use crate::packets::counted;

#[derive(Decode, Debug, Clone)]
pub struct ClientCommand {
    pub action_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct ClientInformation<'a> {
    pub locale: &'a str,
    pub view_distance: i8,
    pub chat_mode: VarInt,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: VarInt,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct CommandSuggestionsRequest<'a> {
    pub transaction_id: VarInt,
    pub text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerAbilities {
    pub flags: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerInput {
    pub sideways: f32,
    pub forward: f32,
    pub flags: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct ResourcePackResponse {
    pub result: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct KeepAlive {
    pub keep_alive_id: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct Pong {
    pub id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct ChangeDifficulty {
    pub difficulty: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct LockDifficulty {
    pub locked: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct PluginMessage<'a> {
    pub channel: &'a str,
    /// The payload has no length prefix: its length is whatever remains in the
    /// packet, so it must be read as the rest of the buffer.
    #[decode(rest)]
    pub data: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct EditBook<'a> {
    pub slot: VarInt,
    pub pages: Vec<&'a str>,
    /// `None` when the client cleared the title.
    pub title: Option<&'a str>,
}

impl<'a> Decode<'a> for EditBook<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let slot = decoder.varint()?;
        let pages = counted(decoder, |d| d.string())?;
        let title = if decoder.bool()? {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self { slot, pages, title })
    }
}
