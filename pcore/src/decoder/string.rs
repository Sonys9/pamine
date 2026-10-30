//! Length-prefixed byte slices and UTF-8 strings.
//!
//! Two framings exist in the protocol:
//!
//! - **VarInt prefix** — every packet field (`String`, `Byte Array`).
//! - **`u16` prefix** — NBT names and the `Array of String` used by
//!   `Update Teams`, which reuse the NBT string framing.

use crate::errors::Error;

use super::decoder::Decoder;

impl<'a> Decoder<'a> {
    /// Reads a VarInt-prefixed byte slice.
    ///
    /// Deliberately not generic: reinterpreting `&[u8]` as `&[u32]` would need
    /// the pointer to be aligned *and* would keep the bytes in native order,
    /// while the protocol is big-endian. Use [`read_type`](Decoder::read_type)
    /// for scalars, or decode element-wise with `from_be_bytes` for arrays.
    pub fn slice(&mut self) -> Result<&'a [u8], Error> {
        let len = self.varint()?.get();
        if len < 0 {
            return Err(Error::Read(0));
        }
        self.read_len(len as usize)
    }

    /// Reads a `u16`-prefixed byte slice.
    ///
    /// This is the NBT framing: a big-endian `u16` length, then the bytes.
    pub fn slice_u16(&mut self) -> Result<&'a [u8], Error> {
        let len = self.u16()? as usize;
        self.read_len(len)
    }

    /// Reads a VarInt-prefixed UTF-8 string, borrowing from the input.
    pub fn string(&mut self) -> Result<&'a str, Error> {
        Self::utf8(self.slice()?)
    }

    /// Reads a `u16`-prefixed UTF-8 string, borrowing from the input.
    pub fn string_u16(&mut self) -> Result<&'a str, Error> {
        Self::utf8(self.slice_u16()?)
    }

    /// Reads a VarInt-prefixed byte array, copying into a `Vec`.
    ///
    /// Prefer [`slice`](Self::slice) when the caller can borrow.
    pub fn bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.varint()?.get();
        if len < 0 {
            return Err(Error::Read(0));
        }
        Ok(self.read_len(len as usize)?.to_vec())
    }

    /// Validates that `bytes` is UTF-8.
    ///
    /// Shared by both string framings so the error mapping lives in one place.
    fn utf8(bytes: &'a [u8]) -> Result<&'a str, Error> {
        std::str::from_utf8(bytes).map_err(|_| Error::Utf8)
    }
}
