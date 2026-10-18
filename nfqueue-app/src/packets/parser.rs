use pcore::{Decode, decoder::Decoder, encoder};

use crate::{buffer::BigBuffer, packets::{c2s::{handshake::{Handshake::Handshaking, self}, packets::C2SPacket, self}, s2c::packets::S2CPacket}};

pub struct Parser {
    state: State,
    pub buffer: BigBuffer,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Handshake,
            buffer: BigBuffer::new()
        }
    }
    
    #[inline]
    pub fn try_parse<'a>(&mut self, side: Side) -> eyre::Result<(Packet<'a>, usize)> {
        let slice: &'a [u8] = unsafe { std::slice::from_raw_parts(self.buffer.buffer.as_ptr(), self.buffer.len) };
        let packet = Decoder::new(slice).slice()?;
        let mut decoder = Decoder::new(packet);
        let opcode = decoder.varint()?.get();
        let diff = packet.len() + encoder::varint(packet.len() as u32).len();
        Ok((
            match self.state {
                State::Handshake => self.handshake_side(opcode, decoder, side)?,
                State::Login => todo!(),
                State::Play => todo!(),
            }, diff
        ))
    }

    fn handshake_side<'a>(&mut self, opcode: i32, mut decoder: Decoder<'a>, side: Side) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::handshake::Handshake::decode_by_opcode(opcode, &mut decoder)?;
                self.state = State::Login;
                Ok(Packet::C2S(C2SPacket::Handshake(parsed)))
            },
            _ => panic!("Impossible event")
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum State {
    Handshake,
    Login,
    Play,
}

#[derive(Clone, Debug)]
pub enum Packet<'a> {
    C2S(C2SPacket<'a>),
    S2C(S2CPacket)
}

pub enum Side {
    C2S,
    S2C
}