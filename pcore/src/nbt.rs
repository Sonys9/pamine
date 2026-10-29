//! Named Binary Tag (NBT) reader.
//!
//! Wire format (all integers big-endian):
//!
//! ```text
//! tag    := type:u8 name:string payload
//! string := len:u16 bytes[len]        // NOT a VarInt
//! list   := elem_type:u8 len:i32 payload[len]
//! compound := (tag)* TAG_End
//! ```
//!
//! `TAG_End` (0) has no name and no payload; it terminates a compound and is
//! also the element type of an empty list.

use std::collections::HashMap;

use crate::decoder::Decoder;
use crate::errors::Error;

/// Maximum nesting depth, matching the vanilla limit.
pub const MAX_DEPTH: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag<'a> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'a [u8]),
    String(&'a str),
    List(Vec<NbtTag<'a>>),
    /// Ordered key/value pairs. NBT compounds preserve insertion order on the
    /// wire, so a `HashMap` would lose information on re-serialization.
    Compound(Vec<(&'a str, NbtTag<'a>)>),
    IntArray(&'a [i32]),
    LongArray(&'a [i64]),
}

impl<'a> NbtTag<'a> {
    /// Looks up a key in a compound. Returns `None` for non-compounds.
    pub fn get(&self, key: &str) -> Option<&NbtTag<'a>> {
        match self {
            NbtTag::Compound(entries) => entries
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, tag)| tag),
            _ => None,
        }
    }

    /// Convenience view of a compound as a map (loses order).
    pub fn as_map(&self) -> Option<HashMap<&'a str, &NbtTag<'a>>> {
        match self {
            NbtTag::Compound(entries) => {
                Some(entries.iter().map(|(k, v)| (*k, v)).collect())
            }
            _ => None,
        }
    }
}

/// Reads a named tag (type + name + payload).
pub fn read_named<'a>(decoder: &mut Decoder<'a>) -> Result<(&'a str, NbtTag<'a>), Error> {
    read_named_at(decoder, 0)
}

/// Reads an unnamed root tag, as used by network packets.
///
/// Network NBT has no root name: the payload starts directly with the type id.
pub fn read_root<'a>(decoder: &mut Decoder<'a>) -> Result<NbtTag<'a>, Error> {
    let type_id = decoder.u8()?;
    if type_id == 0 {
        return Ok(NbtTag::End);
    }
    read_payload(decoder, type_id, 0)
}

fn read_named_at<'a>(
    decoder: &mut Decoder<'a>,
    depth: usize,
) -> Result<(&'a str, NbtTag<'a>), Error> {
    let type_id = decoder.u8()?;
    if type_id == 0 {
        return Ok(("", NbtTag::End));
    }
    let name = read_string(decoder)?;
    let tag = read_payload(decoder, type_id, depth)?;
    Ok((name, tag))
}

fn read_payload<'a>(
    decoder: &mut Decoder<'a>,
    type_id: u8,
    depth: usize,
) -> Result<NbtTag<'a>, Error> {
    if depth > MAX_DEPTH {
        return Err(Error::Nbt("nesting too deep"));
    }

    Ok(match type_id {
        0 => NbtTag::End,
        1 => NbtTag::Byte(decoder.i8()?),
        2 => NbtTag::Short(decoder.i16()?),
        3 => NbtTag::Int(decoder.i32()?),
        4 => NbtTag::Long(decoder.i64()?),
        5 => NbtTag::Float(decoder.f32()?),
        6 => NbtTag::Double(decoder.f64()?),
        7 => {
            let len = read_len(decoder)?;
            NbtTag::ByteArray(decoder.read_len(len)?)
        }
        8 => NbtTag::String(read_string(decoder)?),
        9 => {
            let elem_type = decoder.u8()?;
            let len = read_len(decoder)?;
            // An empty list may carry any element type; TAG_End is the usual one.
            if len == 0 {
                return Ok(NbtTag::List(Vec::new()));
            }
            if elem_type == 0 {
                return Err(Error::Nbt("non-empty list of TAG_End"));
            }
            let mut items = Vec::with_capacity(len.min(1024));
            for _ in 0..len {
                items.push(read_payload(decoder, elem_type, depth + 1)?);
            }
            NbtTag::List(items)
        }
        10 => {
            let mut entries = Vec::new();
            loop {
                let (name, tag) = read_named_at(decoder, depth + 1)?;
                if matches!(tag, NbtTag::End) {
                    break;
                }
                entries.push((name, tag));
            }
            NbtTag::Compound(entries)
        }
        11 => {
            let len = read_len(decoder)?;
            let total = len.checked_mul(4).ok_or(Error::Nbt("int array too large"))?;
            let raw = decoder.read_len(total)?;
            let mut out = Vec::with_capacity(len.min(1024));
            for chunk in raw.chunks_exact(4) {
                out.push(i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
            NbtTag::IntArray(Box::leak(out.into_boxed_slice()))
        }
        12 => {
            let len = read_len(decoder)?;
            let total = len.checked_mul(8).ok_or(Error::Nbt("long array too large"))?;
            let raw = decoder.read_len(total)?;
            let mut out = Vec::with_capacity(len.min(1024));
            for chunk in raw.chunks_exact(8) {
                out.push(i64::from_be_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ]));
            }
            NbtTag::LongArray(Box::leak(out.into_boxed_slice()))
        }
        other => return Err(Error::Nbt("unknown tag type")),
    })
}

/// NBT strings are `u16` big-endian length + UTF-8 bytes, not VarInt-prefixed.
fn read_string<'a>(decoder: &mut Decoder<'a>) -> Result<&'a str, Error> {
    let len = decoder.u16()? as usize;
    let bytes = decoder.read_len(len)?;
    std::str::from_utf8(bytes).map_err(|_| Error::Utf8)
}

/// NBT array/list lengths are `i32` big-endian.
fn read_len(decoder: &mut Decoder<'_>) -> Result<usize, Error> {
    let len = decoder.i32()?;
    if len < 0 {
        return Err(Error::Nbt("negative length"));
    }
    Ok(len as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(bytes: &[u8]) -> Decoder<'_> {
        Decoder::new(bytes)
    }

    #[test]
    fn reads_byte() {
        // type=1, name len=1, "a", value=0x7f
        let data = [1u8, 0, 1, b'a', 0x7f];
        let (name, tag) = read_named(&mut dec(&data)).unwrap();
        assert_eq!(name, "a");
        assert_eq!(tag, NbtTag::Byte(127));
    }

    #[test]
    fn reads_string_with_u16_length() {
        // type=8, name len=1, "s", value len=5, "hello"
        let mut data = vec![8u8, 0, 1, b's', 0, 5];
        data.extend_from_slice(b"hello");
        let (name, tag) = read_named(&mut dec(&data)).unwrap();
        assert_eq!(name, "s");
        assert_eq!(tag, NbtTag::String("hello"));
    }

    #[test]
    fn reads_empty_compound() {
        // type=10, name len=0, then TAG_End
        let data = [10u8, 0, 0, 0];
        let (name, tag) = read_named(&mut dec(&data)).unwrap();
        assert_eq!(name, "");
        assert_eq!(tag, NbtTag::Compound(vec![]));
    }

    #[test]
    fn compound_preserves_order() {
        // {b:1b, a:2b}
        let data = [
            10u8, 0, 0, // compound, empty name
            1, 0, 1, b'b', 1, // b = 1
            1, 0, 1, b'a', 2, // a = 2
            0, // end
        ];
        let (_, tag) = read_named(&mut dec(&data)).unwrap();
        let NbtTag::Compound(entries) = tag else {
            panic!("expected compound");
        };
        assert_eq!(entries[0].0, "b");
        assert_eq!(entries[1].0, "a");
    }

    #[test]
    fn reads_empty_list() {
        // type=9, name len=0, elem_type=0, len=0
        let data = [9u8, 0, 0, 0, 0, 0, 0, 0];
        let (_, tag) = read_named(&mut dec(&data)).unwrap();
        assert_eq!(tag, NbtTag::List(vec![]));
    }

    #[test]
    fn reads_list_of_ints() {
        // type=9, name len=0, elem_type=3, len=2, 1, 2
        let data = [
            9u8, 0, 0, 3, 0, 0, 0, 2, // list header
            0, 0, 0, 1, // 1
            0, 0, 0, 2, // 2
        ];
        let (_, tag) = read_named(&mut dec(&data)).unwrap();
        assert_eq!(tag, NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)]));
    }

    #[test]
    fn reads_byte_array() {
        // type=7, name len=0, len=3, [1,2,3]
        let data = [7u8, 0, 0, 0, 0, 0, 3, 1, 2, 3];
        let (_, tag) = read_named(&mut dec(&data)).unwrap();
        assert_eq!(tag, NbtTag::ByteArray(&[1, 2, 3]));
    }

    #[test]
    fn reads_root_without_name() {
        // network NBT: type=3, value=42
        let data = [3u8, 0, 0, 0, 42];
        assert_eq!(read_root(&mut dec(&data)).unwrap(), NbtTag::Int(42));
    }

    #[test]
    fn rejects_negative_length() {
        // type=7, name len=0, len=-1
        let data = [7u8, 0, 0, 0xff, 0xff, 0xff, 0xff];
        assert!(read_named(&mut dec(&data)).is_err());
    }

    #[test]
    fn rejects_unknown_type() {
        let data = [99u8, 0, 0];
        assert!(read_named(&mut dec(&data)).is_err());
    }

    #[test]
    fn get_looks_up_compound_key() {
        let data = [
            10u8, 0, 0, //
            3, 0, 1, b'x', 0, 0, 0, 7, // x = 7
            0,
        ];
        let (_, tag) = read_named(&mut dec(&data)).unwrap();
        assert_eq!(tag.get("x"), Some(&NbtTag::Int(7)));
        assert_eq!(tag.get("missing"), None);
    }
}
