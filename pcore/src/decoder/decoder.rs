use bytemuck::{Pod, Zeroable};

use crate::errors::Error;

#[derive(Clone, Copy, Debug)]
pub struct Decoder<'a> {
    payload: &'a [u8],
    len: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(payload: &'a [u8]) -> Self { 
        Self { 
            payload,
            len: 0,
        }
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

    /// Reads a VarLong.
    ///
    /// The wire format is a 64-bit two's-complement value encoded 7 bits per
    /// byte, so the accumulated bits are reinterpreted as `i64` at the end.
    pub fn varlong(&mut self) -> Result<VarLong, Error> {
        let mut value: u64 = 0;
        let mut shift = 0;
        for _ in 0..10 {
            let byte = self.u8()?;
            value |= ((byte & 0b0111_1111) as u64) << shift;
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                return Ok(VarLong::new(value as i64));
            }
        }
        Err(Error::VarLong)
    }

    /// Reads a VarInt.
    ///
    /// A VarInt is a 32-bit two's-complement value, so `-1` arrives as
    /// `FF FF FF FF 0F` and must come back as `-1`, not as an error.
    pub fn varint(&mut self) -> Result<VarInt, Error> {
        let mut value: u32 = 0;
        let mut shift = 0;
        for _ in 0..5 {
            let byte = self.u8()?;
            value |= ((byte & 0b0111_1111) as u32) << shift;
            shift += 7;
            if byte & 0b1000_0000 == 0 {
                return Ok(VarInt::new(value as i32));
            }
        }
        Err(Error::VarInt)
    }

    pub fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.read::<u8>()?[0])
    }

    pub fn bool(&mut self) -> Result<bool, Error> {
        Ok(self.u8()? != 0)
    }

    pub fn u16(&mut self) -> Result<u16, Error> {
        Ok(u16::from_be_bytes(self.read::<u16>()?.try_into().unwrap()))
    }

    pub fn i16(&mut self) -> Result<i16, Error> {
        Ok(i16::from_be_bytes(self.read::<i16>()?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_be_bytes(self.read::<u64>()?.try_into().unwrap()))
    }

    pub fn i64(&mut self) -> Result<i64, Error> {
        Ok(i64::from_be_bytes(self.read::<i64>()?.try_into().unwrap()))
    }

    pub fn f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_be_bytes(self.read::<f64>()?.try_into().unwrap()))
    }

    pub fn f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_be_bytes(self.read::<f32>()?.try_into().unwrap()))
    }

    pub fn i32(&mut self) -> Result<i32, Error> {
        Ok(i32::from_be_bytes(self.read::<i32>()?.try_into().unwrap()))
    }

    pub fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_be_bytes(self.read::<u32>()?.try_into().unwrap()))
    }

    pub fn i8(&mut self) -> Result<i8, Error> {
        Ok(i8::from_be_bytes(self.read::<i8>()?.try_into().unwrap()))
    }

    pub fn read_type_ref<T: Pod>(slice: &[u8]) -> Result<&T, Error> {
        bytemuck::try_from_bytes::<T>(slice).map_err(|_| Error::ReadType)
    }

    pub fn read_type<T: Pod>(slice: &[u8]) -> Result<T, Error> {
        bytemuck::try_pod_read_unaligned::<T>(slice).map_err(|_| Error::ReadType)
    }

    pub fn read<T: Pod>(&mut self) -> Result<&'a [u8], Error> {
        let type_len = size_of::<T>();
        self.read_len(type_len)
    }

    pub fn read_len(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.len.checked_add(len).ok_or(Error::Read(len))?;
        if self.payload.len() < end {
            return Err(Error::Read(len));
        }
        let slice = &self.payload[self.len..end];
        self.len = end;
        Ok(slice)
    }

    /// Reads a VarInt-prefixed byte slice.
    ///
    /// Deliberately not generic: reinterpreting `&[u8]` as `&[u32]` would need
    /// the pointer to be aligned *and* would keep the bytes in native order,
    /// while the protocol is big-endian. Use [`read_type`](Self::read_type) for
    /// scalars, or decode element-wise with `from_be_bytes` for arrays.
    pub fn slice(&mut self) -> Result<&'a [u8], Error> {
        let len = self.varint()?.get();
        if len < 0 {
            return Err(Error::Read(0));
        }
        self.read_len(len as usize)
    }

    /// Reads a VarInt-prefixed UTF-8 string.
    pub fn string(&mut self) -> Result<&'a str, Error> {
        let slice = self.slice()?;
        std::str::from_utf8(slice)
            .map_err(|_| Error::Utf8)
    }

    /// Reads a VarInt-prefixed byte array.
    pub fn bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.varint()?.get();
        if len < 0 {
            return Err(Error::Read(0));
        }
        Ok(self.read_len(len as usize)?.to_vec())
    }

    pub fn uuid(&mut self) -> Result<Uuid, Error> {
        Self::read_type::<Uuid>(self.read::<Uuid>()?)
    }

    #[inline]
    pub fn len(&self) -> usize { self.len }

    #[inline]
    pub fn is_empty(&self) -> bool { self.len == 0 }

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

#[derive(Debug, Clone)]
pub struct VarInt {
    pub number: i32
}

impl VarInt {
    pub fn new(number: i32) -> Self { Self { number } }
    pub fn edit(&mut self, number: i32) { self.number = number }
    pub fn get(&self) -> i32 { self.number }
}

#[derive(Debug, Clone)]
pub struct VarLong {
    pub number: i64
}

impl VarLong {
    pub fn new(number: i64) -> Self { Self { number } }
    pub fn edit(&mut self, number: i64) { self.number = number }
    pub fn get(&self) -> i64 { self.number }
}

#[derive(Debug, Clone, Copy)]
pub struct Uuid {
    pub bytes: [u8; 16]
}

unsafe impl Zeroable for Uuid {}

unsafe impl Pod for Uuid {}

impl Uuid {
    pub fn new(bytes: [u8; 16]) -> Self { Self { bytes } }
    pub fn edit(&mut self, bytes: [u8; 16]) { self.bytes = bytes }
    pub fn get(&self) -> [u8; 16] { self.bytes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_positive() {
        let mut d = Decoder::new(&[0x00]);
        assert_eq!(d.varint().unwrap().get(), 0);

        let mut d = Decoder::new(&[0x7f]);
        assert_eq!(d.varint().unwrap().get(), 127);

        let mut d = Decoder::new(&[0xac, 0x02]);
        assert_eq!(d.varint().unwrap().get(), 300);
    }

    #[test]
    fn varint_negative() {
        // -1 == FF FF FF FF 0F
        let mut d = Decoder::new(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert_eq!(d.varint().unwrap().get(), -1);

        // -2147483648 == 80 80 80 80 08
        let mut d = Decoder::new(&[0x80, 0x80, 0x80, 0x80, 0x08]);
        assert_eq!(d.varint().unwrap().get(), i32::MIN);
    }

    #[test]
    fn varint_max() {
        // i32::MAX == FF FF FF FF 07
        let mut d = Decoder::new(&[0xff, 0xff, 0xff, 0xff, 0x07]);
        assert_eq!(d.varint().unwrap().get(), i32::MAX);
    }

    #[test]
    fn varlong_negative() {
        // -1 as VarLong == FF FF FF FF FF FF FF FF FF 01
        let mut d = Decoder::new(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]);
        assert_eq!(d.varlong().unwrap().get(), -1);
    }

    #[test]
    fn varint_too_long_errors() {
        let mut d = Decoder::new(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80]);
        assert!(d.varint().is_err());
    }

    #[test]
    fn get_does_not_panic_past_end() {
        let mut d = Decoder::new(&[1, 2, 3]);
        d.read_len(3).unwrap();
        assert_eq!(d.get(), &[] as &[u8]);
        assert_eq!(d.remaining(), 0);
    }

    #[test]
    fn read_len_rejects_overflow() {
        let mut d = Decoder::new(&[1, 2, 3]);
        assert!(d.read_len(usize::MAX).is_err());
    }

    #[test]
    fn set_rewinds_position() {
        let mut d = Decoder::new(&[1, 2, 3]);
        d.read_len(2).unwrap();
        assert_eq!(d.len(), 2);
        d.set(&[9, 9]);
        assert_eq!(d.len(), 0);
        assert_eq!(d.u8().unwrap(), 9);
    }

    #[test]
    fn rest_consumes_tail() {
        let mut d = Decoder::new(&[1, 2, 3, 4]);
        d.read_len(1).unwrap();
        assert_eq!(d.rest(), &[2, 3, 4]);
        assert_eq!(d.remaining(), 0);
    }

    #[test]
    fn string_roundtrip() {
        // len=2, "hi"
        let mut d = Decoder::new(&[0x02, b'h', b'i']);
        assert_eq!(d.string().unwrap(), "hi");
    }

    #[test]
    fn bytes_roundtrip() {
        let mut d = Decoder::new(&[0x03, 1, 2, 3]);
        assert_eq!(d.bytes().unwrap(), vec![1, 2, 3]);
    }
}