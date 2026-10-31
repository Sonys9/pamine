//! Byte-level encoding for the Minecraft protocol.
//!
//! This is the mirror of [`decoder`](crate::decoder): every reader there has a
//! writer here. The `write_*` functions append to a caller-owned `Vec<u8>`, so
//! encoding a packet body does not allocate a temporary buffer per field.
//!
//! Module layout mirrors the decoder:
//!
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
//! The older `varint`/`varlong`/`string`/`bytes` helpers return a fresh `Vec`
//! and are kept for callers that want a standalone value.

pub mod frame;
pub mod nbt;
pub mod primitives;
pub mod slot;
pub mod string;
pub mod uuid;
pub mod varint;

#[cfg(test)]
mod tests;

pub use frame::packet_frame;
pub use string::{bytes, string, write_bytes, write_string, write_string_u16};
pub use varint::{varint, varlong, write_varint, write_varlong};
