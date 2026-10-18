pub struct Decoder<'a> {
    payload: &'a [u8],
}

impl<'a> Decoder<'a> {
    pub fn new(payload: &'a [u8]) -> Self { Self { payload } }

    pub fn varint(&mut self) -> eyre::Result<VarInt> {
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
        eyre::bail!("Invalid varint")
    }

    pub fn u8(&mut self) -> eyre::Result<u8> {
        Ok(self.read(1)?[0])
    }

    pub fn u16(&mut self) -> eyre::Result<u16> {
        Ok(u16::from_be_bytes(self.read(2)?.try_into().unwrap()))
    }

    pub fn i32(&mut self) -> eyre::Result<i32> {
        Ok(i32::from_be_bytes(self.read(4)?.try_into().unwrap()))
    }

    fn read(&mut self, len: usize) -> eyre::Result<&'a [u8]> {
        if self.payload.len() < len {
            eyre::bail!("Failed to get a slice because the payload is too small! ({} < {})", self.payload.len(), len);
        };
        let slice = &self.payload[..len];
        self.payload = &self.payload[len..];
        Ok(slice)
    }

    pub fn slice(&mut self) -> eyre::Result<&[u8]> {
        let len = self.varint()?.get() as usize;
        self.read(len)
    }
}

pub struct VarInt {
    pub number: i32
}

impl VarInt {
    pub fn new(number: i32) -> Self { Self { number } }
    pub fn edit(&mut self, number: i32) { self.number = number }
    pub fn get(&self) -> i32 { self.number }
}