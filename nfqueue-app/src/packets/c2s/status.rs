use pcore::Decode;
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Status {
    #[id(0x00)]
    StatusRequest(StatusRequest),
    #[id(0x01)]
    PingRequest(PingRequest),
}

#[derive(Decode, Debug, Clone)]
pub struct StatusRequest {}

#[derive(Decode, Debug, Clone)]
pub struct PingRequest {
    pub payload: i64,
}