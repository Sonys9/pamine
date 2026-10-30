pub mod c2s;
pub mod parser;
pub mod s2c;

pub use parser::*;
use pcore::{
    decoder::{Decoder, MAX_ELEMENTS},
    errors::Error,
};

/// Reads an "Optional Byte Array": a VarInt length where `0` means absent.
fn optional_bytes<'a>(decoder: &mut Decoder<'a>) -> Result<Option<&'a [u8]>, Error> {
    let bytes = decoder.slice()?;
    Ok(if bytes.is_empty() { None } else { Some(bytes) })
}

/// Reads a VarInt-counted array, decoding each element with `read`.
///
/// The count is validated against [`MAX_ELEMENTS`] before allocating, so a
/// hostile packet cannot ask for a multi-gigabyte `Vec`.
fn counted<'a, T>(
    decoder: &mut Decoder<'a>,
    read: impl Fn(&mut Decoder<'a>) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let count = decoder.varint()?.get();
    if count < 0 {
        return Err(Error::Read(0));
    }
    let count = count as usize;
    if count > MAX_ELEMENTS {
        return Err(Error::LenLimit(count));
    }
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(read(decoder)?);
    }
    Ok(out)
}

/// Reads an "Optional Chat": a `bool` gate followed by a JSON string.
fn optional_chat<'a>(decoder: &mut Decoder<'a>) -> Result<Option<&'a str>, Error> {
    Ok(if decoder.bool()? {
        Some(decoder.string()?)
    } else {
        None
    })
}

/// Reads a VarInt-counted array of Identifiers (VarInt-prefixed strings).
fn identifiers<'a>(decoder: &mut Decoder<'a>) -> Result<Vec<&'a str>, Error> {
    counted(decoder, |d| d.string())
}
