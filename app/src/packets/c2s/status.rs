//! Serverbound status packets (protocol 760 / 1.19.2).

use pcore::Decode;
use pcore_gen::{DecodePacket, Encode, EncodePacket};

#[derive(DecodePacket, EncodePacket, Clone, Debug)]
pub enum Status {
    #[id(0x00)]
    StatusRequest(StatusRequest),
    #[id(0x01)]
    PingRequest(PingRequest),
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct StatusRequest {}

#[derive(Decode, Encode, Debug, Clone)]
pub struct PingRequest {
    pub payload: i64,
}
