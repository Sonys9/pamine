//! Core cursor over a byte slice.
//!
//! The type-specific readers live in sibling modules, each adding an `impl`
//! block: [`varint`](super::varint), [`primitives`](super::primitives),
//! [`string`](super::string), [`uuid`](super::uuid) and [`nbt`](super::nbt).

use bytemuck::Pod;

use crate::errors::Error;

#[derive(Clone, Copy, Debug)]
pub struct Decoder<'a> {
    payload: &'a [u8],
    len: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(payload: &'a [u8]) -> Self {
        Self { payload, len: 0 }
    }

    /// Returns the not-yet-consumed tail. Never panics.
    pub fn get(&self) -> &'a [u8] {
        self.payload.get(self.len..).unwrap_or(&[])
    }

    /// Replaces the payload and rewinds the read position.
    pub fn set(&mut self, payload: &'a [u8]) {
        self.payload = payload;
        self.len = 0;
    }

    /// Borrows `T` from `slice` without copying. Requires alignment.
    pub fn read_type_ref<T: Pod>(slice: &[u8]) -> Result<&T, Error> {
        bytemuck::try_from_bytes::<T>(slice).map_err(|_| Error::ReadType)
    }

    /// Reads `T` from `slice`, tolerating misalignment.
    pub fn read_type<T: Pod>(slice: &[u8]) -> Result<T, Error> {
        bytemuck::try_pod_read_unaligned::<T>(slice).map_err(|_| Error::ReadType)
    }

    /// Advances by `size_of::<T>()` and returns the raw bytes.
    pub fn read<T: Pod>(&mut self) -> Result<&'a [u8], Error> {
        let type_len = size_of::<T>();
        self.read_len(type_len)
    }

    /// Advances by `len` and returns the raw bytes.
    pub fn read_len(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.len.checked_add(len).ok_or(Error::Read(len))?;
        if self.payload.len() < end {
            return Err(Error::Read(len));
        }
        let slice = &self.payload[self.len..end];
        self.len = end;
        Ok(slice)
    }

    /// Bytes consumed so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Number of bytes still available to read.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.payload.len().saturating_sub(self.len)
    }

    /// Consumes and returns everything that is left.
    pub fn rest(&mut self) -> &'a [u8] {
        let tail = self.get();
        self.len = self.payload.len();
        tail
    }
}
