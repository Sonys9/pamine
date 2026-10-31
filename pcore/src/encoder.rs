//! Byte-level encoding for the Minecraft protocol.
//!
//! This is the mirror of [`decoder`](crate::decoder): every reader there has a
//! writer here. The `write_*` functions append to a caller-owned `Vec<u8>`, so
//! encoding a packet body does not allocate a temporary buffer per field.
//!
//! The older `varint`/`varlong`/`string`/`bytes` helpers return a fresh `Vec`
//! and are kept for callers that want a standalone value.

use crate::decoder::{NbtTag, Slot, Uuid, VarInt, VarLong};

/// Appends a signed VarInt (32-bit two's-complement, 7 bits per byte).
pub fn write_varint(out: &mut Vec<u8>, number: i32) {
    let mut value = number as u32;
    loop {
        let byte = (value & 0b0111_1111) as u8;
        value >>= 7;
        if value != 0 {
            out.push(byte | 0b1000_0000);
        } else {
            out.push(byte);
            break;
        }
    }
}

/// Appends a signed VarLong (64-bit two's-complement, 7 bits per byte).
pub fn write_varlong(out: &mut Vec<u8>, number: i64) {
    let mut value = number as u64;
    loop {
        let byte = (value & 0b0111_1111) as u8;
        value >>= 7;
        if value != 0 {
            out.push(byte | 0b1000_0000);
        } else {
            out.push(byte);
            break;
        }
    }
}

/// Appends a VarInt-prefixed UTF-8 string.
pub fn write_string(out: &mut Vec<u8>, value: &str) {
    write_varint(out, value.len() as i32);
    out.extend_from_slice(value.as_bytes());
}

/// Appends a `u16`-prefixed UTF-8 string (the NBT framing).
pub fn write_string_u16(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u16).to_be_bytes());
    out.extend_from_slice(value.as_bytes());
}

/// Appends a VarInt-prefixed byte array.
pub fn write_bytes(out: &mut Vec<u8>, value: &[u8]) {
    write_varint(out, value.len() as i32);
    out.extend_from_slice(value);
}

/// Encodes a signed VarInt into a fresh `Vec`.
pub fn varint(number: i32) -> Vec<u8> {
    let mut out = Vec::with_capacity(5);
    write_varint(&mut out, number);
    out
}

/// Encodes a signed VarLong into a fresh `Vec`.
pub fn varlong(number: i64) -> Vec<u8> {
    let mut out = Vec::with_capacity(10);
    write_varlong(&mut out, number);
    out
}

/// Encodes a VarInt-prefixed UTF-8 string into a fresh `Vec`.
pub fn string(value: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len() + 5);
    write_string(&mut out, value);
    out
}

/// Encodes a VarInt-prefixed byte array into a fresh `Vec`.
pub fn bytes(value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len() + 5);
    write_bytes(&mut out, value);
    out
}

/// Wraps a packet body (id + data) in the uncompressed frame format.
pub fn packet_frame(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 5);
    write_varint(&mut out, body.len() as i32);
    out.extend_from_slice(body);
    out
}

// ---------------------------------------------------------------------------
// `Encode` impls for the protocol's own types
// ---------------------------------------------------------------------------

/// Fixed-width big-endian scalars, mirroring the decoder's readers.
macro_rules! impl_encode_scalar {
    ($($ty:ty),* $(,)?) => {
        $(
            impl crate::Encode for $ty {
                fn encode(&self, out: &mut Vec<u8>) {
                    out.extend_from_slice(&self.to_be_bytes());
                }
            }
        )*
    };
}

impl_encode_scalar!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

impl crate::Encode for bool {
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(*self as u8);
    }
}

impl crate::Encode for VarInt {
    fn encode(&self, out: &mut Vec<u8>) {
        write_varint(out, self.get());
    }
}

impl crate::Encode for VarLong {
    fn encode(&self, out: &mut Vec<u8>) {
        write_varlong(out, self.get());
    }
}

impl crate::Encode for Uuid {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.bytes);
    }
}

impl crate::Encode for str {
    fn encode(&self, out: &mut Vec<u8>) {
        write_string(out, self);
    }
}

/// References encode exactly like the value they point at.
///
/// This is what lets the derive macro pass `&self.field` for every field
/// without caring whether the field is already a reference.
impl<T: crate::Encode + ?Sized> crate::Encode for &T {
    fn encode(&self, out: &mut Vec<u8>) {
        (**self).encode(out);
    }
}

impl<T: crate::Encode + ?Sized> crate::Encode for &mut T {
    fn encode(&self, out: &mut Vec<u8>) {
        (**self).encode(out);
    }
}

impl crate::Encode for String {
    fn encode(&self, out: &mut Vec<u8>) {
        write_string(out, self);
    }
}

impl crate::Encode for [u8] {
    fn encode(&self, out: &mut Vec<u8>) {
        write_bytes(out, self);
    }
}

impl<T: crate::Encode> crate::Encode for Option<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Some(value) => {
                out.push(1);
                value.encode(out);
            }
            None => out.push(0),
        }
    }
}

/// A counted array: VarInt length, then each element.
///
/// This also covers `Vec<u8>`, which is the same wire shape as a byte array —
/// a VarInt length followed by the bytes.
impl<T: crate::Encode> crate::Encode for Vec<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        write_varint(out, self.len() as i32);
        for value in self {
            value.encode(out);
        }
    }
}

impl crate::Encode for Slot<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        if !self.present {
            out.push(0);
            return;
        }
        out.push(1);
        // A present slot always carries an id, a count and an NBT tag; the
        // decoder fills all three together, so the fallbacks are unreachable.
        match &self.item_id {
            Some(id) => id.encode(out),
            None => write_varint(out, 0),
        }
        out.push(self.count.unwrap_or(0) as u8);
        match &self.nbt {
            Some(nbt) => nbt.encode(out),
            None => out.push(0),
        }
    }
}

impl crate::Encode for NbtTag<'_> {
    fn encode(&self, out: &mut Vec<u8>) {
        // A root tag carries an empty name in 1.19.2.
        encode_named(out, "", self);
    }
}

/// Writes one named NBT tag: type id, `u16` name length, name, payload.
fn encode_named(out: &mut Vec<u8>, name: &str, tag: &NbtTag<'_>) {
    out.push(tag_id(tag));
    if !matches!(tag, NbtTag::End) {
        write_string_u16(out, name);
    }
    encode_payload(out, tag);
}

/// The wire type id of a tag.
fn tag_id(tag: &NbtTag<'_>) -> u8 {
    match tag {
        NbtTag::End => 0,
        NbtTag::Byte(_) => 1,
        NbtTag::Short(_) => 2,
        NbtTag::Int(_) => 3,
        NbtTag::Long(_) => 4,
        NbtTag::Float(_) => 5,
        NbtTag::Double(_) => 6,
        NbtTag::ByteArray(_) => 7,
        NbtTag::String(_) => 8,
        NbtTag::List(_) => 9,
        NbtTag::Compound(_) => 10,
        NbtTag::IntArray(_) => 11,
        NbtTag::LongArray(_) => 12,
    }
}

/// Writes a tag's payload, without the type id and name.
fn encode_payload(out: &mut Vec<u8>, tag: &NbtTag<'_>) {
    match tag {
        NbtTag::End => {}
        NbtTag::Byte(v) => out.push(*v as u8),
        NbtTag::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Float(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::Double(v) => out.extend_from_slice(&v.to_be_bytes()),
        NbtTag::ByteArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            out.extend_from_slice(v);
        }
        NbtTag::String(v) => write_string_u16(out, v),
        NbtTag::List(items) => {
            // An empty list still needs an element type; TAG_End is the usual one.
            let elem = items.first().map(tag_id).unwrap_or(0);
            out.push(elem);
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            for item in items {
                encode_payload(out, item);
            }
        }
        NbtTag::Compound(entries) => {
            for (name, value) in entries {
                encode_named(out, name, value);
            }
            out.push(0); // TAG_End terminates the compound
        }
        NbtTag::IntArray(values) => {
            out.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for v in values {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
        NbtTag::LongArray(values) => {
            out.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for v in values {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;
    use crate::{Decode, Encode};

    #[test]
    fn varint_roundtrip_positive() {
        for n in [0, 1, 127, 128, 300, 25565, i32::MAX] {
            let encoded = varint(n);
            let mut d = Decoder::new(&encoded);
            assert_eq!(d.varint().unwrap().get(), n, "failed for {n}");
        }
    }

    #[test]
    fn varint_roundtrip_negative() {
        for n in [-1, -2, -128, -300, i32::MIN] {
            let encoded = varint(n);
            let mut d = Decoder::new(&encoded);
            assert_eq!(d.varint().unwrap().get(), n, "failed for {n}");
        }
    }

    #[test]
    fn varlong_roundtrip() {
        for n in [0i64, 1, -1, i64::MAX, i64::MIN, 1234567890123] {
            let encoded = varlong(n);
            let mut d = Decoder::new(&encoded);
            assert_eq!(d.varlong().unwrap().get(), n, "failed for {n}");
        }
    }

    #[test]
    fn varint_known_encoding() {
        assert_eq!(varint(0), vec![0x00]);
        assert_eq!(varint(127), vec![0x7f]);
        assert_eq!(varint(128), vec![0x80, 0x01]);
        assert_eq!(varint(300), vec![0xac, 0x02]);
        assert_eq!(varint(-1), vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
    }

    #[test]
    fn string_roundtrip() {
        let encoded = string("hello");
        let mut d = Decoder::new(&encoded);
        assert_eq!(d.string().unwrap(), "hello");
    }

    #[test]
    fn bytes_roundtrip() {
        let encoded = bytes(&[1, 2, 3]);
        let mut d = Decoder::new(&encoded);
        assert_eq!(d.bytes().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn packet_frame_roundtrip() {
        let body = b"\x00\x01\x02";
        let framed = packet_frame(body);
        let mut d = Decoder::new(&framed);
        let len = d.varint().unwrap().get() as usize;
        assert_eq!(len, body.len());
        assert_eq!(d.rest(), body);
    }

    #[test]
    fn write_helpers_append() {
        let mut out = vec![0xAA];
        write_varint(&mut out, 300);
        write_string(&mut out, "hi");
        assert_eq!(out, vec![0xAA, 0xac, 0x02, 0x02, b'h', b'i']);
    }

    #[test]
    fn nbt_roundtrip_compound() {
        let tag = NbtTag::Compound(vec![
            ("a", NbtTag::Byte(1)),
            ("b", NbtTag::Int(256)),
            ("c", NbtTag::String("hi")),
        ]);
        let mut out = Vec::new();
        tag.encode(&mut out);

        let mut d = Decoder::new(&out);
        assert_eq!(d.nbt().unwrap(), tag);
        assert_eq!(d.remaining(), 0);
    }

    #[test]
    fn nbt_roundtrip_arrays() {
        for tag in [
            NbtTag::ByteArray(&[1, 2, 3]),
            NbtTag::IntArray(vec![1, -2, 3]),
            NbtTag::LongArray(vec![1, -2, 3]),
            NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)]),
            NbtTag::List(vec![]),
            NbtTag::End,
        ] {
            let mut out = Vec::new();
            tag.encode(&mut out);
            let mut d = Decoder::new(&out);
            assert_eq!(d.nbt().unwrap(), tag, "failed for {tag:?}");
            assert_eq!(d.remaining(), 0, "trailing bytes for {tag:?}");
        }
    }

    #[test]
    fn slot_roundtrip() {
        let slot = Slot {
            present: true,
            item_id: Some(VarInt::new(5)),
            count: Some(3),
            nbt: Some(NbtTag::End),
        };
        let mut out = Vec::new();
        slot.encode(&mut out);

        let mut d = Decoder::new(&out);
        let decoded = Slot::decode(&mut d).unwrap();
        assert_eq!(decoded, slot);
        assert_eq!(d.remaining(), 0);
    }

    #[test]
    fn empty_slot_roundtrip() {
        let slot = Slot::empty();
        let mut out = Vec::new();
        slot.encode(&mut out);
        assert_eq!(out, vec![0]);

        let mut d = Decoder::new(&out);
        assert_eq!(Slot::decode(&mut d).unwrap(), slot);
    }
}
