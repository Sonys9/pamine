use pcore::{
    Decode,
    decoder::{Uuid, VarInt},
};
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Login<'a> {
    #[id(0x00)] Disconnect(Disconnect<'a>),
    #[id(0x01)] EncryptionRequest(EncryptionRequest<'a>),
    #[id(0x02)] LoginSuccess(LoginSuccess<'a>),
    #[id(0x03)] SetCompression(SetCompression),
    #[id(0x04)] LoginPluginRequest(LoginPluginRequest<'a>),
}

#[derive(Decode, Debug, Clone)]
pub struct Disconnect<'a> { pub reason: &'a [u8] }

#[derive(Decode, Debug, Clone)]
pub struct EncryptionRequest<'a> { pub server_id: &'a [u8], pub public_key: &'a [u8], pub verify_token: &'a [u8] }

#[derive(Decode, Debug, Clone)]
pub struct LoginSuccess<'a> { pub uuid: Uuid, pub username: &'a [u8], pub number_of_properties: VarInt }

#[derive(Decode, Debug, Clone)]
pub struct SetCompression { pub threshold: VarInt }

#[derive(Decode, Debug, Clone)]
pub struct LoginPluginRequest<'a> { pub message_id: VarInt, pub channel: &'a [u8], pub data: &'a [u8] }