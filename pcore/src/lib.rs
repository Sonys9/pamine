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

pub use pcore_gen::Decode;
