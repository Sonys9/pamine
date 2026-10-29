use std::{borrow::Cow, io::Read, mem, slice::from_raw_parts};

use crate::{
    buffer::BigBuffer,
    packets::{
        c2s::{self, packets::C2SPacket},
        s2c::{self, packets::S2CPacket},
    },
};
use flate2::read::{GzDecoder, ZlibDecoder};
use pcore::decoder::Decoder;
use tracing::info;

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
    threshold: Option<i32>,
    decompress_buffer: Vec<u8>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Handshake,
            c2s: StreamData::default(),
            s2c: StreamData::default(),
            threshold: None,
            decompress_buffer: Vec::new(),
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub fn try_parse<'a>(&'a mut self, side: Side) -> eyre::Result<Packet<'a>> {
        let stream_data = Self::sd_by_side(side, &mut self.c2s, &mut self.s2c);
        let slice: &'a [u8] = &stream_data.buffer.buffer[stream_data.diff..stream_data.buffer.len];
        let (mut decoder, consumed) =
            Self::decode_or_pass(slice, self.threshold, &mut self.decompress_buffer)?;
        let Ok(opcode) = decoder.varint().map(|opcode| opcode.get()) else {
            eyre::bail!("Bad packet opcode");
        };
        // Advance by exactly one packet, not by the whole remaining buffer.
        stream_data.diff += consumed;
        match self.state {
            State::Handshake => Self::handshake_side(&mut self.state, opcode, decoder, side),
            State::Login => {
                Self::login_side(&mut self.state, opcode, decoder, side, &mut self.threshold)
            }
            State::Play => Self::play_side(opcode, decoder, side),
            State::Status => Self::status_side(opcode, decoder, side),
        }
    }

    /// Splits one packet off the front of `slice`.
    ///
    /// Returns a decoder positioned at the packet body (opcode first) plus the
    /// number of bytes consumed from `slice`, so the caller can advance its
    /// read cursor by exactly one packet.
    #[inline(always)]
    fn decode_or_pass<'a>(
        slice: &'a [u8],
        threshold: Option<i32>,
        decompress_buffer: &'a mut Vec<u8>,
    ) -> eyre::Result<(Decoder<'a>, usize)> {
        let mut head = Decoder::new(slice);
        let len = head.varint()?.get();
        if len < 0 {
            eyre::bail!("negative packet length: {len}");
        }
        let len = len as usize;
        // Bytes actually consumed by the length prefix (handles non-canonical VarInts).
        let len_bytes_len = head.len();
        let total = len
            .checked_add(len_bytes_len)
            .ok_or_else(|| eyre::eyre!("packet length overflow"))?;
        if slice.len() < total {
            eyre::bail!("packet is too small! {} < {}", slice.len(), total);
        }

        let body = &slice[len_bytes_len..total];
        let mut decoder = Decoder::new(body);

        let Some(threshold) = threshold else {
            return Ok((decoder, total));
        };

        let data_len = decoder.varint()?.get();
        if data_len <= 0 {
            // Uncompressed packet inside a compressed stream.
            return Ok((decoder, total));
        }

        decompress_buffer.clear();
        let len = Self::decode(decoder.get(), decompress_buffer)?;
        Ok((Decoder::new(&decompress_buffer[..len]), total))
    }

    #[inline(always)]
    fn decode<'a>(
        packet: &[u8],
        decompress_buffer: &'a mut Vec<u8>,
    ) -> Result<usize, std::io::Error> {
        let mut decoder = ZlibDecoder::new(packet);
        decoder.read_to_end(decompress_buffer)
    }

    #[inline(always)]
    fn login_side<'a>(
        state: &mut State,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
        threshold: &mut Option<i32>,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                Ok(Packet::C2S(C2SPacket::Login(parsed)))
            }
            Side::S2C => {
                let parsed = s2c::login::Login::decode_by_opcode(opcode, &mut decoder)?;
                match parsed {
                    s2c::login::Login::LoginSuccess(_) => *state = State::Play,
                    s2c::login::Login::SetCompression(ref set_compression) => {
                        *threshold = Some(set_compression.threshold.get())
                    }
                    _ => {}
                };
                Ok(Packet::S2C(S2CPacket::Login(parsed)))
            }
        }
    }

    #[inline(always)]
    fn handshake_side<'a>(
        state: &mut State,
        opcode: i32,
        mut decoder: Decoder<'a>,
        side: Side,
    ) -> eyre::Result<Packet<'a>> {
        match side {
            Side::C2S => {
                let parsed = c2s::handshake::Handshake::decode_by_opcode(opcode, &mut decoder)?;
                let c2s::handshake::Handshake::Handshaking(ref packet) = parsed;
                match packet.next_state.get() {
                    1 => *state = State::Status,
                    _ => *state = State::Login,
                };
                Ok(Packet::C2S(C2SPacket::Handshake(parsed)))
            }
            Side::S2C => panic!("Impossible event"),
        }
    }

    #[inline(always)]
    fn status_side<'a>(
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
            }
        }
    }

    #[inline(always)]
    fn play_side<'a>(
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
            }
        }
    }

    #[inline(always)]
    pub fn clear(&mut self, side: Side) {
        let stream_data = Self::sd_by_side(side, &mut self.c2s, &mut self.s2c);
        stream_data.buffer.move_buffer(stream_data.diff, 0);
        stream_data.diff = 0;
        if !self.decompress_buffer.is_empty() {
            self.decompress_buffer.clear();
        }
    }

    #[inline(always)]
    pub fn is_clear(&mut self, side: Side) -> bool {
        Self::sd_by_side(side, &mut self.c2s, &mut self.s2c)
            .buffer
            .len
            == 0
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
            diff: 0, // last_seq: 0,
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
