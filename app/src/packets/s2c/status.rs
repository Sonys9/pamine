//! Clientbound status packets (protocol 760 / 1.19.2).

use pcore::Decode;
use pcore_gen::{DecodePacket, Encode, EncodePacket, Lua};

#[derive(DecodePacket, EncodePacket, Lua, Clone, Debug)]
pub enum Status<'a> {
    #[id(0x00)]
    StatusResponse(StatusResponse<'a>),
    #[id(0x01)]
    PingResponse(PingResponse),
}

#[derive(Decode, Encode, Lua, Debug, Clone)]
pub struct StatusResponse<'a> {
    pub json_response: &'a str,
}

#[derive(Decode, Encode, Lua, Debug, Clone)]
pub struct PingResponse {
    pub payload: i64,
}
