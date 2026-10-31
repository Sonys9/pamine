//! VarInt / VarLong writers.
//!
//! Both are two's-complement values encoded 7 bits per byte, least significant
//! group first, with the high bit of each byte marking continuation.

use crate::decoder::{VarInt, VarLong};

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

impl crate::Encode for VarInt {
    fn encode(&self, encoder: &mut crate::encoder::Encoder) {
        encoder.varint(self.get());
    }
}

impl crate::Encode for VarLong {
    fn encode(&self, encoder: &mut crate::encoder::Encoder) {
        encoder.varlong(self.get());
    }
}
