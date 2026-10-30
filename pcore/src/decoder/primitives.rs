//! Fixed-width big-endian scalar readers.
//!
//! Every integer and float on the Minecraft wire is big-endian, so these all go
//! through `from_be_bytes` rather than a raw reinterpret.

use crate::errors::Error;

use super::decoder::Decoder;

impl<'a> Decoder<'a> {
    pub fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.read::<u8>()?[0])
    }

    pub fn i8(&mut self) -> Result<i8, Error> {
        Ok(i8::from_be_bytes(self.read::<i8>()?.try_into().unwrap()))
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

    pub fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_be_bytes(self.read::<u32>()?.try_into().unwrap()))
    }

    pub fn i32(&mut self) -> Result<i32, Error> {
        Ok(i32::from_be_bytes(self.read::<i32>()?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_be_bytes(self.read::<u64>()?.try_into().unwrap()))
    }

    pub fn i64(&mut self) -> Result<i64, Error> {
        Ok(i64::from_be_bytes(self.read::<i64>()?.try_into().unwrap()))
    }

    pub fn f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_be_bytes(self.read::<f32>()?.try_into().unwrap()))
    }

    pub fn f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_be_bytes(self.read::<f64>()?.try_into().unwrap()))
    }
}
