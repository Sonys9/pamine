//! VarInt-prefixed byte slices and UTF-8 strings.

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

    /// Reads a VarInt-prefixed UTF-8 string, borrowing from the input.
    pub fn string(&mut self) -> Result<&'a str, Error> {
        let slice = self.slice()?;
        std::str::from_utf8(slice).map_err(|_| Error::Utf8)
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
}
