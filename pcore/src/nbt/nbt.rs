use crate::{decoder::Decoder, errors::Error};

/// Maximum nesting depth for `List` and `Compound`, matching the vanilla limit.
/// Without it a malicious payload of nested `0x0A` bytes would blow the stack.
pub const MAX_DEPTH: usize = 512;

/// Maximum number of elements in a single `List` or array.
///
/// The length arrives as an `i32` from the wire, so a hostile packet can claim
/// two billion elements. Rejecting up front avoids a huge `Vec::with_capacity`
/// before any data has actually been read.
pub const MAX_ELEMENTS: usize = 1 << 20; // 1_048_576

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
    Compound(Vec<(&'a str, NbtTag<'a>)>), // Not hashmap because we dont need hashing
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl<'a> NbtTag<'a> {
    /// Parses a root tag from `src`.
    ///
    /// Network NBT has no root name: the payload starts with the type id.
    pub fn decode(src: &'a [u8]) -> Result<Self, Error> {
        let mut decoder = Decoder::new(src);
        let id = decoder.u8()?; // u8, not VarInt
        Self::parse(id, &mut decoder, 0)
    }

    /// Parses the payload of a tag whose type id has already been read.
    ///
    /// `decoder` is taken by mutable reference: `Decoder` is `Copy`, so passing
    /// it by value would advance a copy and leave the caller's cursor untouched.
    fn parse(id: u8, decoder: &mut Decoder<'a>, depth: usize) -> Result<Self, Error> {
        if depth > MAX_DEPTH {
            return Err(Error::Nbt("nesting too deep"));
        }

        Ok(match id {
            0 => Self::End,
            1 => Self::Byte(decoder.i8()?),
            2 => Self::Short(decoder.i16()?),
            3 => Self::Int(decoder.i32()?),
            4 => Self::Long(decoder.i64()?),
            5 => Self::Float(decoder.f32()?),
            6 => Self::Double(decoder.f64()?),
            7 => {
                let len = Self::len(decoder)?;
                Self::ByteArray(decoder.read_len(len)?)
            }
            8 => Self::String(Self::string(decoder)?),
            9 => {
                let elements_id = decoder.u8()?;
                let len = Self::len(decoder)?;
                // An empty list may carry any element type; TAG_End is the usual one.
                if len == 0 {
                    return Ok(Self::List(Vec::new()));
                }
                if elements_id == 0 {
                    return Err(Error::Nbt("non-empty list of TAG_End"));
                }
                let mut array = Vec::with_capacity(len);
                for _ in 0..len {
                    array.push(Self::parse(elements_id, decoder, depth + 1)?);
                }
                Self::List(array)
            }
            10 => {
                let mut compound = Vec::new();
                loop {
                    let id = decoder.u8()?;
                    if id == 0 {
                        // TAG_End terminates the compound.
                        break;
                    }
                    let name = Self::string(decoder)?;
                    let nbt = Self::parse(id, decoder, depth + 1)?;
                    compound.push((name, nbt));
                }
                Self::Compound(compound)
            }
            11 => {
                let len = Self::len(decoder)?;
                let mut numbers: Vec<i32> = Vec::with_capacity(len);
                for _ in 0..len {
                    numbers.push(decoder.i32()?);
                }
                // Cannot borrow as &[i32]: the wire is big-endian and the pointer
                // is not guaranteed to be aligned.
                Self::IntArray(numbers)
            }
            12 => {
                let len = Self::len(decoder)?;
                let mut numbers: Vec<i64> = Vec::with_capacity(len);
                for _ in 0..len {
                    numbers.push(decoder.i64()?);
                }
                Self::LongArray(numbers)
            }
            _ => return Err(Error::Nbt("unknown tag type")),
        })
    }

    /// Reads an `i32` big-endian length and validates it against [`MAX_ELEMENTS`].
    fn len(decoder: &mut Decoder<'a>) -> Result<usize, Error> {
        let len = decoder.i32()?;
        if len < 0 {
            return Err(Error::Nbt("negative length"));
        }
        let len = len as usize;
        if len > MAX_ELEMENTS {
            return Err(Error::LenLimit(len));
        }
        Ok(len)
    }

    /// NBT strings are `u16` big-endian length + UTF-8 bytes, not VarInt-prefixed.
    fn string(decoder: &mut Decoder<'a>) -> Result<&'a str, Error> {
        let len = decoder.u16()? as usize;
        str::from_utf8(decoder.read_len(len)?).map_err(|_| Error::Utf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_byte() {
        // type=1, value=0x7f
        assert_eq!(NbtTag::decode(&[1, 0x7f]).unwrap(), NbtTag::Byte(127));
    }

    #[test]
    fn reads_short_and_int() {
        assert_eq!(NbtTag::decode(&[2, 0x01, 0x00]).unwrap(), NbtTag::Short(256));
        assert_eq!(
            NbtTag::decode(&[3, 0x00, 0x00, 0x01, 0x00]).unwrap(),
            NbtTag::Int(256)
        );
    }

    #[test]
    fn reads_string_with_u16_length() {
        let mut data = vec![8u8, 0, 5];
        data.extend_from_slice(b"hello");
        assert_eq!(NbtTag::decode(&data).unwrap(), NbtTag::String("hello"));
    }

    #[test]
    fn reads_empty_compound() {
        assert_eq!(NbtTag::decode(&[10, 0]).unwrap(), NbtTag::Compound(vec![]));
    }

    #[test]
    fn compound_preserves_order_and_advances() {
        // {b:1b, a:2b}
        let data = [
            10u8, //
            1, 0, 1, b'b', 1, // b = 1
            1, 0, 1, b'a', 2, // a = 2
            0, // end
        ];
        let NbtTag::Compound(entries) = NbtTag::decode(&data).unwrap() else {
            panic!("expected compound");
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "b");
        assert_eq!(entries[0].1, NbtTag::Byte(1));
        assert_eq!(entries[1].0, "a");
        assert_eq!(entries[1].1, NbtTag::Byte(2));
    }

    #[test]
    fn reads_empty_list() {
        // type=9, elem_type=0, len=0
        assert_eq!(
            NbtTag::decode(&[9, 0, 0, 0, 0, 0]).unwrap(),
            NbtTag::List(vec![])
        );
    }

    #[test]
    fn reads_list_of_ints() {
        // type=9, elem_type=3, len=2, 1, 2
        let data = [
            9u8, 3, 0, 0, 0, 2, // header
            0, 0, 0, 1, // 1
            0, 0, 0, 2, // 2
        ];
        assert_eq!(
            NbtTag::decode(&data).unwrap(),
            NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)])
        );
    }

    #[test]
    fn reads_byte_array() {
        // type=7, len=3, [1,2,3]
        assert_eq!(
            NbtTag::decode(&[7, 0, 0, 0, 3, 1, 2, 3]).unwrap(),
            NbtTag::ByteArray(&[1, 2, 3])
        );
    }

    #[test]
    fn reads_int_array() {
        // type=11, len=2, 1, 2
        let data = [11u8, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2];
        assert_eq!(
            NbtTag::decode(&data).unwrap(),
            NbtTag::IntArray(vec![1, 2])
        );
    }

    #[test]
    fn reads_long_array() {
        // type=12, len=1, 1
        let mut data = vec![12u8, 0, 0, 0, 1];
        data.extend_from_slice(&1i64.to_be_bytes());
        assert_eq!(NbtTag::decode(&data).unwrap(), NbtTag::LongArray(vec![1]));
    }

    #[test]
    fn rejects_negative_length() {
        // type=7, len=-1
        assert!(NbtTag::decode(&[7, 0xff, 0xff, 0xff, 0xff]).is_err());
    }

    #[test]
    fn rejects_length_over_limit() {
        // type=7, len = MAX_ELEMENTS + 1
        let len = (MAX_ELEMENTS as i32) + 1;
        let mut data = vec![7u8];
        data.extend_from_slice(&len.to_be_bytes());
        assert!(matches!(
            NbtTag::decode(&data),
            Err(Error::LenLimit(_))
        ));
    }

    #[test]
    fn rejects_unknown_type() {
        assert!(NbtTag::decode(&[99]).is_err());
    }

    #[test]
    fn rejects_non_empty_list_of_end() {
        // type=9, elem_type=0, len=1
        assert!(NbtTag::decode(&[9, 0, 0, 0, 0, 1]).is_err());
    }

    #[test]
    fn rejects_excessive_nesting() {
        // MAX_DEPTH + 2 nested compounds, each with an empty name, never closed.
        let mut data = vec![10u8];
        for _ in 0..=MAX_DEPTH {
            data.extend_from_slice(&[10, 0, 0]); // compound, empty name
        }
        assert!(matches!(NbtTag::decode(&data), Err(Error::Nbt(_))));
    }
}
