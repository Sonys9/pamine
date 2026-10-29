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

    pub fn get(&self) -> &'a [u8] { &self.payload[self.len..] }
    pub fn set(&mut self, payload: &'a [u8]) { self.payload = payload }

    pub fn varlong(&mut self) -> Result<VarLong, Error> {
        let mut value = 0;
        let mut shift = 0;
        for _ in 0..10 {
            let byte = self.u8()?;
            value |= (byte as i64 & 0b1111111) << shift;
            shift += 7;
            if byte & 0b10000000 == 0 {
                return Ok(VarLong::new(value));
            };
        };
        Err(Error::VarLong)
    }

    pub fn varint(&mut self) -> Result<VarInt, Error> {
        let number = self.varlong()?.get();
        if number > i32::MAX as i64 {
            return Err(Error::VarInt);
        };
        Ok(VarInt::new(number as i32))
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
        if self.payload.len() < self.len + len {
            return Err(Error::Read(len))
        };
        let slice = &self.payload[self.len..self.len + len];
        self.len += len;
        Ok(slice)
    }
    
    pub fn slice<T: Pod>(&mut self) -> Result<&'a [T], Error> {
        let len = self.varint()?.get() as usize;
        let total = len * size_of::<T>();
        let slice = bytemuck::try_cast_slice::<u8, T>(self.read_len(total)?).map_err(|_| Error::Read(total))?; // so it converts &[u8] to &[T]
        Ok(slice)
    }

    /// Reads a VarInt-prefixed UTF-8 string.
    pub fn string(&mut self) -> Result<String, Error> {
        let len = self.varint()?.get() as usize;
        let bytes = self.read_len(len)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Error::Utf8)
    }

    /// Reads a VarInt-prefixed byte array.
    pub fn bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.varint()?.get() as usize;
        Ok(self.read_len(len)?.to_vec())
    }

    pub fn uuid(&mut self) -> Result<Uuid, Error> {
        Self::read_type::<Uuid>(self.read::<Uuid>()?)
    }

    #[inline]
    pub fn len(&self) -> usize { self.len }
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