//! 128-bit UUID reader.

use bytemuck::{Pod, Zeroable};

use crate::errors::Error;

use super::decoder::Decoder;

impl<'a> Decoder<'a> {
    /// Reads a 16-byte UUID.
    pub fn uuid(&mut self) -> Result<Uuid, Error> {
        Self::read_type::<Uuid>(self.read::<Uuid>()?)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Uuid {
    pub bytes: [u8; 16],
}

// SAFETY: `Uuid` is `[u8; 16]` with no padding, so every bit pattern is valid.
unsafe impl Zeroable for Uuid {}
unsafe impl Pod for Uuid {}

impl Uuid {
    pub fn new(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }
    pub fn edit(&mut self, bytes: [u8; 16]) {
        self.bytes = bytes;
    }
    pub fn get(&self) -> [u8; 16] {
        self.bytes
    }
}
