//! VarInt / VarLong readers.
//!
//! Both are two's-complement values encoded 7 bits per byte, least significant
//! group first, with the high bit of each byte marking continuation.

use crate::errors::Error;

use super::decoder::Decoder;

impl<'a> Decoder<'a> {
    /// Reads a VarLong.
    ///
    /// The wire format is a 64-bit two's-complement value, so the accumulated
    /// bits are reinterpreted as `i64` at the end.
    pub fn varlong(&mut self) -> Result<VarLong, Error> {
        let mut value: u64 = 0;
        let mut shift = 0;
        for _ in 0..10 {
            let byte = self.u8()?;
            value |= ((byte & 0b0111_1111) as u64) << shift;
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                return Ok(VarLong::new(value as i64));
            }
        }
        Err(Error::VarLong)
    }

    /// Reads a VarInt.
    ///
    /// A VarInt is a 32-bit two's-complement value, so `-1` arrives as
    /// `FF FF FF FF 0F` and must come back as `-1`, not as an error.
    pub fn varint(&mut self) -> Result<VarInt, Error> {
        let mut value: u32 = 0;
        let mut shift = 0;
        for _ in 0..5 {
            let byte = self.u8()?;
            value |= ((byte & 0b0111_1111) as u32) << shift;
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                return Ok(VarInt::new(value as i32));
            }
        }
        Err(Error::VarInt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarInt {
    pub number: i32,
}

impl VarInt {
    pub fn new(number: i32) -> Self {
        Self { number }
    }
    pub fn edit(&mut self, number: i32) {
        self.number = number;
    }
    pub fn get(&self) -> i32 {
        self.number
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarLong {
    pub number: i64,
}

impl VarLong {
    pub fn new(number: i64) -> Self {
        Self { number }
    }
    pub fn edit(&mut self, number: i64) {
        self.number = number;
    }
    pub fn get(&self) -> i64 {
        self.number
    }
}
