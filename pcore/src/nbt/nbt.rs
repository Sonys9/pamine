use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum NbtTag<'a> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'a [u8]), // 4 byte len + slice
    String(&'a str), // 2 byte slice len + slice
    List(Vec<NbtTag<'a>>),
    Compound(HashMap<&'a str, NbtTag<'a>>), // 4 byte len + slice
    IntArray(&'a [i32]), // 4 byte len + slice
    LongArray(&'a [i64]), // 4 byte len + slice
}

impl<'a> TryFrom<u8> for NbtTag<'a> {
    type Error = crate::errors::Error;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        todo!("map NBT tag id {id} to a variant")
    }
}

impl<'a> NbtTag<'a> {
    fn decode(src: &[u8]) -> Self {
        todo!("decode NBT from {src:?}")
    }
}