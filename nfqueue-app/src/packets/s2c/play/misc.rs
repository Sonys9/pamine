//! Packets that do not fit a larger subject group.

use pcore::{
    Decode,
    decoder::{NbtTag, VarInt},
};

#[derive(Decode, Debug, Clone)]
pub struct PluginMessage<'a> {
    pub channel: &'a str,
    /// Everything after the channel: the payload has no length prefix, its
    /// length is whatever remains in the packet. This is genuinely opaque —
    /// the layout is defined by the plugin, not by the protocol.
    #[decode(rest)]
    pub data: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct Ping {
    pub id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct KeepAlive {
    pub keep_alive_id: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct ServerData<'a> {
    pub has_motd: bool,
    pub motd: Option<&'a str>,
    pub has_icon: bool,
    pub icon: Option<&'a str>,
    pub previews_chat: bool,
    pub enforces_secure_chat: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct ResourcePack<'a> {
    pub url: &'a str,
    pub hash: &'a str,
    pub forced: bool,
    pub has_prompt_message: bool,
    pub prompt_message: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct TagQueryResponse<'a> {
    pub transaction_id: VarInt,
    pub nbt: NbtTag<'a>,
}
