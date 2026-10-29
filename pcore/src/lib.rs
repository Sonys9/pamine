use crate::errors::Error;

pub mod decoder;
pub mod errors;
pub mod encoder;
pub mod nbt;

pub trait Decode<'a>: Sized {
    fn decode(decoder: &mut decoder::Decoder<'a>) -> Result<Self, Error>;
}

pub use pcore_gen::Decode;