//! Byte-level encoding for the Minecraft protocol.
//!
//! This is the mirror of [`decoder`](crate::decoder). Where the decoder is a
//! cursor over a borrowed slice, [`Encoder`] is a cursor over an owned buffer,
//! and every reader on `Decoder` has a writer with the same name:
//!
//! ```text
//! Decoder::new(&bytes)   Encoder::new()
//! decoder.varint()?      encoder.varint(300)
//! decoder.string()?      encoder.string("hi")
//! decoder.rest()         encoder.raw(&[1, 2, 3])
//! ```
//!
//! Module layout mirrors the decoder:
//!
//! - [`encoder`] — the cursor itself: `new`, `into_bytes`, `as_slice`, `len`,
//!   `clear`, and the `varint`/`string`/`nbt`/… writers
//! - [`varint`] — `write_varint`, `write_varlong` and the `Encode` impls for
//!   [`VarInt`](crate::decoder::VarInt) / [`VarLong`](crate::decoder::VarLong)
//! - [`primitives`] — fixed-width big-endian scalars and `bool`
//! - [`string`] — VarInt- and `u16`-prefixed strings, byte arrays, `Vec<T>`,
//!   `Option<T>` and the reference forwarding impls
//! - [`uuid`] — 16-byte UUID
//! - [`slot`] — inventory slot (`bool + VarInt id + i8 count + NBT`)
//! - [`nbt`] — Named Binary Tag, including the 1.19.2 root name
//! - [`frame`] — the uncompressed packet length prefix
//!
//! The free `varint`/`varlong`/`string`/`bytes` helpers return a fresh `Vec`
//! and are kept for callers that want a standalone value.

pub mod encoder;
pub mod frame;
pub mod nbt;
pub mod primitives;
pub mod slot;
pub mod string;
pub mod uuid;
pub mod varint;

#[cfg(test)]
mod tests;

pub use encoder::{Encoder, Scalar};
pub use frame::packet_frame;
pub use string::{bytes, string, write_bytes, write_string, write_string_u16};
pub use varint::{varint, varlong, write_varint, write_varlong};
