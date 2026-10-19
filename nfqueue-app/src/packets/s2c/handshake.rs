use pcore::Decode;
use pcore_gen::DecodePacket;

#[derive(Clone, Debug)]
pub enum Handshake {}

#[derive(Decode, Debug, Clone)]
pub struct HandshakeOk;
