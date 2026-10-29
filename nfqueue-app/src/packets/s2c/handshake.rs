use pcore::Decode;

#[derive(Clone, Debug)]
pub enum Handshake {}

#[derive(Decode, Debug, Clone)]
pub struct HandshakeOk;
