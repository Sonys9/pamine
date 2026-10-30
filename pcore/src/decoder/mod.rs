//! Byte-level decoding for the Minecraft protocol.
//!
//! [`Decoder`] is a cursor over a borrowed byte slice. The type-specific readers
//! are split across modules, each contributing an `impl` block:
//!
//! - [`decoder`] — the cursor itself: `new`, `get`, `set`, `read`, `read_len`,
//!   `rest`, `remaining`
//! - [`varint`] — `varint`, `varlong` and their wrapper types
//! - [`primitives`] — fixed-width big-endian scalars (`u8`…`f64`, `bool`)
//! - [`string`] — VarInt-prefixed `&[u8]`, `&str` and `Vec<u8>`
//! - [`uuid`] — 16-byte UUID
//! - [`nbt`] — Named Binary Tag, self-describing on the wire

pub mod decoder;
pub mod nbt;
pub mod primitives;
pub mod string;
pub mod uuid;
pub mod varint;

#[cfg(test)]
mod tests;

pub use decoder::Decoder;
pub use nbt::{MAX_DEPTH, MAX_ELEMENTS, NbtTag};
pub use uuid::Uuid;
pub use varint::{VarInt, VarLong};
