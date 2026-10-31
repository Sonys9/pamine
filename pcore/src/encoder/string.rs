//! Length-prefixed byte slices and UTF-8 strings.
//!
//! Two framings exist in the protocol, matching the decoder:
//!
//! - **VarInt prefix** — every packet field (`String`, `Byte Array`).
//! - **`u16` prefix** — NBT names and NBT strings.

use super::varint::write_varint;

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

impl crate::Encode for str {
    fn encode(&self, out: &mut Vec<u8>) {
        write_string(out, self);
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

/// A boolean-gated value: `1` then the payload, or a lone `0`.
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
