//! Serverbound login packets (protocol 760 / 1.19.2).

use pcore::{
    Decode,
    decoder::{Uuid, VarInt},
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
    pub has_sig_data: bool,
    pub sig_data: Option<SigData<'a>>,
    pub has_player_uuid: bool,
    pub player_uuid: Option<Uuid>,
}

#[derive(Decode, Debug, Clone)]
pub struct SigData<'a> {
    pub timestamp: i64,
    pub public_key: &'a [u8],
    pub signature: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct EncryptionResponse<'a> {
    pub shared_secret: &'a [u8],
    pub has_verify_token: bool,
    pub verify_token: Option<&'a [u8]>,
    pub salt: Option<i64>,
    pub message_signature: Option<&'a [u8]>,
}

#[derive(Decode, Debug, Clone)]
pub struct LoginPluginResponse<'a> {
    pub message_id: VarInt,
    pub successful: bool,
    pub data: Option<&'a [u8]>,
}
