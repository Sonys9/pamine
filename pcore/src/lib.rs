use crate::errors::Error;

pub mod decoder;
pub mod encoder;
pub mod errors;

pub trait Decode<'a>: Sized {
    fn decode(decoder: &mut decoder::Decoder<'a>) -> Result<Self, Error>;
}

impl<'a> Decode<'a> for decoder::NbtTag<'a> {
    fn decode(decoder: &mut decoder::Decoder<'a>) -> Result<Self, Error> {
        decoder.nbt()
    }
}

/// Writes a value into a packet body.
///
/// The mirror of [`Decode`]: every type that can be read off the wire can be
/// written back. Implementations append to `out` instead of returning a fresh
/// `Vec`, so encoding a struct with many fields does not allocate per field.
pub trait Encode {
    fn encode(&self, out: &mut Vec<u8>);
}

/// Encodes a value into a fresh `Vec`.
pub fn encode<T: Encode + ?Sized>(value: &T) -> Vec<u8> {
    let mut out = Vec::new();
    value.encode(&mut out);
    out
}

pub use pcore_gen::{Decode, Encode};
