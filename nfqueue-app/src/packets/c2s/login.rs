use pcore::{Decode, decoder::Uuid};
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
    pub name: &'a [u8],
    pub sig_data: Option<SigData<'a>>,
    pub uuid: Option<Uuid>,
}

#[derive(Decode, Debug, Clone)]
pub struct SigData<'a> {
    pub timestamp: u64,
    pub public_key: &'a [u8],
    pub signature: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct EncryptionResponse<'a> {
    pub shared_secret: &'a [u8],
    pub verify_token: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct LoginPluginResponse<'a> {
    pub message_id: pcore::decoder::VarInt,
    pub successful: bool,
    pub data: Option<&'a [u8]>,
}
