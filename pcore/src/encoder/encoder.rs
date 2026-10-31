//! The write cursor.
//!
//! The mirror of [`Decoder`](crate::decoder::Decoder): where the decoder is a
//! cursor over a borrowed slice, the encoder is a cursor over an owned buffer.
//! Every reader on `Decoder` has a writer here with the same name.
//!
//! ```text
//! Decoder::new(&bytes)      Encoder::new()
//! decoder.varint()?         encoder.varint(300)
//! decoder.string()?         encoder.string("hi")
//! decoder.rest()            encoder.raw(&[1, 2, 3])
//! ```
//!
//! Writing cannot fail — the buffer grows as needed — so the methods return
//! `()` instead of `Result`.

use crate::decoder::{NbtTag, Slot, Uuid};

use super::nbt::write_nbt;
use super::primitives::write_scalar;
use super::slot::write_slot;
use super::string::{write_bytes, write_string, write_string_u16};
use super::varint::{write_varint, write_varlong};

/// An owned byte buffer that protocol values are appended to.
#[derive(Debug, Default, Clone)]
pub struct Encoder {
    out: Vec<u8>,
}

impl Encoder {
    /// An empty encoder.
    pub fn new() -> Self {
        Self { out: Vec::new() }
    }

    /// An empty encoder with room for `capacity` bytes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            out: Vec::with_capacity(capacity),
        }
    }

    /// Consumes the encoder and returns the bytes written so far.
    pub fn into_bytes(self) -> Vec<u8> {
        self.out
    }

    /// The bytes written so far.
    pub fn as_slice(&self) -> &[u8] {
        &self.out
    }

    /// Number of bytes written so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.out.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.out.is_empty()
    }

    /// Drops everything written so far, keeping the allocation.
    pub fn clear(&mut self) {
        self.out.clear();
    }

    /// Appends a signed VarInt.
    pub fn varint(&mut self, number: i32) {
        write_varint(&mut self.out, number);
    }

    /// Appends a signed VarLong.
    pub fn varlong(&mut self, number: i64) {
        write_varlong(&mut self.out, number);
    }

    /// Appends a VarInt-prefixed UTF-8 string.
    pub fn string(&mut self, value: &str) {
        write_string(&mut self.out, value);
    }

    /// Appends a `u16`-prefixed UTF-8 string (the NBT framing).
    pub fn string_u16(&mut self, value: &str) {
        write_string_u16(&mut self.out, value);
    }

    /// Appends a VarInt-prefixed byte array.
    pub fn bytes(&mut self, value: &[u8]) {
        write_bytes(&mut self.out, value);
    }

    /// Appends raw bytes with no length prefix.
    ///
    /// This is the counterpart of [`Decoder::rest`](crate::decoder::Decoder::rest)
    /// and of `#[decode(rest)]`.
    pub fn raw(&mut self, value: &[u8]) {
        self.out.extend_from_slice(value);
    }

    /// Appends a fixed-width big-endian scalar.
    pub fn scalar<T: Scalar>(&mut self, value: T) {
        write_scalar(&mut self.out, value);
    }

    pub fn u8(&mut self, value: u8) {
        self.out.push(value);
    }

    pub fn i8(&mut self, value: i8) {
        self.out.push(value as u8);
    }

    pub fn bool(&mut self, value: bool) {
        self.out.push(value as u8);
    }

    pub fn u16(&mut self, value: u16) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    pub fn i16(&mut self, value: i16) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    pub fn u32(&mut self, value: u32) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    pub fn i32(&mut self, value: i32) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    pub fn u64(&mut self, value: u64) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    pub fn i64(&mut self, value: i64) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    pub fn f32(&mut self, value: f32) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    pub fn f64(&mut self, value: f64) {
        self.out.extend_from_slice(&value.to_be_bytes());
    }

    /// Appends a 16-byte UUID.
    pub fn uuid(&mut self, value: &Uuid) {
        self.out.extend_from_slice(&value.bytes);
    }

    /// Appends an NBT tag, including the 1.19.2 root name.
    pub fn nbt(&mut self, value: &NbtTag<'_>) {
        write_nbt(&mut self.out, value);
    }

    /// Appends an inventory slot.
    pub fn slot(&mut self, value: &Slot<'_>) {
        write_slot(&mut self.out, value);
    }

    /// Appends any value that implements [`Encode`](crate::Encode).
    pub fn encode<T: crate::Encode + ?Sized>(&mut self, value: &T) {
        value.encode(self);
    }
}

/// A fixed-width big-endian scalar.
///
/// Implemented for every integer and float the protocol uses, so
/// [`Encoder::scalar`] can stay generic without a macro at the call site.
pub trait Scalar: Copy {
    fn write_to(self, out: &mut Vec<u8>);
}

macro_rules! impl_scalar {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Scalar for $ty {
                fn write_to(self, out: &mut Vec<u8>) {
                    out.extend_from_slice(&self.to_be_bytes());
                }
            }
        )*
    };
}

impl_scalar!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);
