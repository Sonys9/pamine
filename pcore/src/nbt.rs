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