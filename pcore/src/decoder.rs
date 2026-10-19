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

    pub fn varint(&mut self) -> Result<VarInt, Error> {
        let mut value: i32 = 0;
        let mut shift = 0;
        for _ in 0..5 {
            let byte = self.u8()?;
            value |= (byte as i32 & 0b1111111) << shift;
            shift += 7;
            if byte & 0b10000000 == 0 {
                return Ok(VarInt::new(value));
            };
        };
        Err(Error::VarInt)
    }

    pub fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.read(1)?[0])
    }

    pub fn bool(&mut self) -> Result<bool, Error> {
        Ok(self.u8()? != 0)
    }

    pub fn u16(&mut self) -> Result<u16, Error> {
        Ok(u16::from_be_bytes(self.read(2)?.try_into().unwrap()))
    }

    pub fn i16(&mut self) -> Result<i16, Error> {
        Ok(i16::from_be_bytes(self.read(2)?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_be_bytes(self.read(8)?.try_into().unwrap()))
    }

    pub fn i64(&mut self) -> Result<i64, Error> {
        Ok(i64::from_be_bytes(self.read(8)?.try_into().unwrap()))
    }

    pub fn f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_be_bytes(self.read(8)?.try_into().unwrap()))
    }

    pub fn f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_be_bytes(self.read(4)?.try_into().unwrap()))
    }

    pub fn i32(&mut self) -> Result<i32, Error> {
        Ok(i32::from_be_bytes(self.read(4)?.try_into().unwrap()))
    }

    pub fn i8(&mut self) -> Result<i8, Error> {
        Ok(i8::from_be_bytes(self.read(1)?.try_into().unwrap()))
    }

    fn read(&mut self, len: usize) -> Result<&'a [u8], Error> {
        self.len += len;
        if self.payload.len() < len {
            return Err(Error::Read(len))
        };
        let slice = &self.payload[..len];
        self.payload = &self.payload[len..];
        Ok(slice)
    }

    pub fn slice(&mut self) -> Result<&'a [u8], Error> {
        let len = self.varint()?.get() as usize;
        self.read(len)
    }

    pub fn uuid(&mut self) -> Result<&'a [u8], Error> {
        self.read(16)
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
pub struct Uuid {
    pub bytes: [u8; 16]
}

impl Uuid {
    pub fn new(bytes: [u8; 16]) -> Self { Self { bytes } }
    pub fn edit(&mut self, bytes: [u8; 16]) { self.bytes = bytes }
    pub fn get(&self) -> [u8; 16] { self.bytes }
}