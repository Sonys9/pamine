use pcore::{decoder::Decoder, encoder};
use crate::{
    buffer::BigBuffer,
    packets::{
        c2s::{self, packets::C2SPacket},
        s2c::{self, packets::S2CPacket},
    },
};

/*pub struct Session<'a> {
    side: Side,
    stream_data: &'a mut StreamData,
    diff: usize,
    state: &'a mut State,
}

impl<'a> Session<'a> {
    pub fn new(side: Side, stream_data: &'a mut StreamData, state: &'a mut State) -> Self {
        Self {
            side,
            stream_data,
            diff: 0,
            state,
        }
    }

    #[inline]
    pub fn try_parse(&mut self) -> eyre::Result<Packet<'a>> {
        let slice: &'a [u8] = unsafe {
            std::slice::from_raw_parts(
                self.stream_data.buffer.buffer.as_ptr(),
                self.stream_data.buffer.len,
            )
        };
        let Ok(packet) = Decoder::new(slice).slice() else {
            eyre::bail!("Bad packet");
        };
        let mut decoder = Decoder::new(packet);
        let Ok(opcode) = decoder.varint() else {
            eyre::bail!("Bad packet opcode");
        };
        self.diff += packet.len() + encoder::varint(packet.len() as u32).len();
        match self.state {
            State::Handshake => self.handshake_side(opcode.get(), decoder, self.side),
            State::Login => self.login_side(opcode.get(), decoder, self.side),
            State::Play => todo!(),
        }
    }

    #[inline]
    fn login_side(
        &mut self,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Login(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                *self.state = State::Play;
                Ok(Packet::S2C(S2CPacket::Login(parsed)))
            }
        }
    }

    #[inline]
    fn handshake_side(
        &mut self,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::handshake::Handshake::decode_by_opcode(opcode, &mut decoder)?;
                *self.state = State::Login;
                Ok(Packet::C2S(C2SPacket::Handshake(parsed)))
            }
            Side::S2C => panic!("Impossible event"),
        }
    }

impl<'a> Drop for Session<'a> {
    fn drop(&mut self) {
        self.stream_data.buffer.move_buffer(self.diff, 0);
    }
}

}*/



pub struct Parser {
    state: State,
    pub c2s: StreamData,
    pub s2c: StreamData,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Handshake,
            c2s: StreamData::default(),
            s2c: StreamData::default(),
        }
    }

    #[inline]
    pub fn update(&mut self, side: Side, slice: &[u8]) {
        let stream_data = Self::sd_by_side(side, &mut self.c2s, &mut self.s2c);
        // if stream_data.last_seq >= seq {
        //     return;
        // };
        stream_data.buffer.update(slice, slice.len());
    }

    /*#[inline]
    pub fn session<'a>(&'a mut self, side: Side) -> Session<'a> {
        Session::new(
            side,
            Self::sd_by_side(side, &mut self.c2s, &mut self.s2c),
            &mut self.state,
        )
    }*/

    #[inline]
    pub fn sd_by_side<'a>(
        side: Side,
        c2s: &'a mut StreamData,
        s2c: &'a mut StreamData,
    ) -> &'a mut StreamData {
        match side {
            Side::C2S => c2s,
            Side::S2C => s2c,
        }
    }

    #[inline]
    pub fn try_parse<'a>(&mut self, side: Side) -> eyre::Result<Packet<'a>> {
        let stream_data = Self::sd_by_side(side, &mut self.c2s, &mut self.s2c);
        let slice: &'a [u8] = unsafe {
            std::slice::from_raw_parts(
                stream_data.buffer.buffer.as_ptr(),
                stream_data.buffer.len,
            )
        };
        let Ok(packet) = Decoder::new(slice).slice() else {
            eyre::bail!("Bad packet");
        };
        let mut decoder = Decoder::new(packet);
        let Ok(opcode) = decoder.varint() else {
            eyre::bail!("Bad packet opcode");
        };
        stream_data.diff += packet.len() + encoder::varint(packet.len() as u32).len();
        match self.state {
            State::Handshake => self.handshake_side(opcode.get(), decoder, side),
            State::Login => self.login_side(opcode.get(), decoder, side),
            State::Play => self.play_side(opcode.get(), decoder, side),
            State::Status => self.status_side(opcode.get(), decoder, side),
        }
    }

    #[inline]
    fn login_side<'a>(
        &mut self,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Login(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                self.state = State::Play;
                Ok(Packet::S2C(S2CPacket::Login(parsed)))
            }
        }
    }

    #[inline]
    fn handshake_side<'a>(
        &mut self,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::handshake::Handshake::decode_by_opcode(opcode, &mut decoder)?;
                self.state = State::Login;
                Ok(Packet::C2S(C2SPacket::Handshake(parsed)))
            }
            Side::S2C => panic!("Impossible event"),
        }
    }

    #[inline]
    fn status_side<'a>(
        &mut self,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::status::Status::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Status(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::status::Status::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::S2C(S2CPacket::Status(parsed)))
            },
        }
    }

    #[inline]
    fn play_side<'a>(
        &mut self,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::play::Play::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Play(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::play::Play::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::S2C(S2CPacket::Play(parsed)))
            },
        }
    }

    #[inline]
    pub fn clear(&mut self, side: Side) {
        let stream_data = Self::sd_by_side(side, &mut self.c2s, &mut self.s2c);
        stream_data.buffer.move_buffer(stream_data.diff, 0);
    }

    /*
    #[inline]
    pub fn reset(&mut self) {
        self.state = State::Handshake;
        self.c2s.reset();
        self.s2c.reset();
    }
    */
}

pub struct StreamData {
    pub buffer: BigBuffer,
    diff: usize,
    // pub last_seq: u32,
}

/*
impl StreamData {
    pub fn reset(&mut self) {
        self.buffer.len = 0;
        // self.last_seq = 0;
    }
}
*/

impl Default for StreamData {
    fn default() -> Self {
        Self {
            buffer: BigBuffer::new(),
            diff: 0
            // last_seq: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum State {
    Handshake,
    Login,
    Play,
    Status,
}

#[derive(Clone, Debug)]
pub enum Packet<'a> {
    C2S(C2SPacket<'a>),
    S2C(S2CPacket<'a>),
}

#[derive(Clone, Copy, Debug)]
pub enum Side {
    C2S,
    S2C,
}
