#[derive(Debug)]
pub enum Handshake<'a> {
    Handshake {
        protocol: VarInt,
        address: &'a str,
        port: u16,
        next_state: VarInt,
    }
}

impl<'a> Handshake<'a> {
    
}

pub fn parse<'a>(packet: &'a [u8]) {

}
