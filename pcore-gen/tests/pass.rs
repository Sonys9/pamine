//! Positive tests - these should compile successfully

use pcore::decoder::{NbtTag, Uuid, VarInt, VarLong};
use pcore_gen::{Decode, DecodePacket, Encode, EncodePacket};

// Test basic struct with primitive types (only those supported by decoder)
#[derive(Decode)]
struct BasicStruct {
    a: i8,
    b: u8,
    c: bool,
    d: i16,
    e: u16,
    f: i32,
    g: i64,
    h: u64,
    i: f32,
    j: f64,
}

// Test struct with Option
#[derive(Decode)]
struct WithOption {
    required: i32,
    optional: Option<i32>,
}

// Test struct with reference to slice
#[derive(Decode)]
struct WithSlice<'a> {
    data: &'a [u8],
    count: VarInt,
}

// Test struct with VarInt/VarLong/Uuid
#[derive(Decode)]
struct WithSpecialTypes {
    varint: VarInt,
    varlong: VarLong,
    uuid: Uuid,
}

// Test unit struct
#[derive(Decode)]
struct UnitStruct;

// Test tuple struct (single field)
#[derive(Decode)]
struct TupleStruct(i32);

// Test tuple struct with multiple fields
#[derive(Decode)]
struct MultiTupleStruct(i32, i64, bool);

// Test struct with String and Vec<u8>
#[derive(Decode)]
struct WithOwned<'a> {
    name: &'a str,
    data: Vec<u8>,
}

// Test borrowed &str - must not allocate
#[derive(Decode)]
struct WithBorrowedStr<'a> {
    name: &'a str,
    id: VarInt,
}

// Test &str with an explicitly named lifetime that differs from 'a
#[derive(Decode)]
struct WithRenamedLifetime<'b> {
    label: &'b str,
}

// Test &str alongside &[u8] in the same struct
#[derive(Decode)]
struct WithStrAndSlice<'a> {
    name: &'a str,
    payload: &'a [u8],
}

// Test struct with u32
#[derive(Decode)]
struct WithU32 {
    value: u32,
}

// Test generic struct with lifetime
#[derive(Decode)]
struct GenericStruct<'a> {
    data: &'a [u8],
    id: VarInt,
}

// Test NBT field - the tag type id is on the wire, so no extra prefix
#[derive(Decode)]
struct WithNbt<'a> {
    id: VarInt,
    data: NbtTag<'a>,
}

// Test NBT inside a packet variant
#[derive(DecodePacket)]
enum PacketWithNbt<'a> {
    #[id(0)]
    WithNbt(WithNbt<'a>),
    #[id(1)]
    Empty,
}

// Test enum with DecodePacket - only unnamed single field and unit variants
#[derive(DecodePacket)]
enum PacketEnum {
    #[id(0)]
    Variant1(i32),
    #[id(1)]
    Variant2,
    #[id(2)]
    Variant3(VarInt),
}

// Test enum with DecodePacket - struct variant with named fields
#[derive(DecodePacket)]
enum PacketEnumWithStructVariant {
    #[id(10)]
    Simple(i32),
    #[id(11)]
    Complex { x: i32, y: i64 },
    #[id(12)]
    Empty,
}

// Test enum with DecodePacket - multiple unit variants
#[derive(DecodePacket)]
enum PacketEnumMultipleUnits {
    #[id(100)]
    Ping,
    #[id(101)]
    Pong,
    #[id(102)]
    Disconnect,
}

// ---------------------------------------------------------------------------
// Encode
// ---------------------------------------------------------------------------

// Every primitive the decoder supports must also be encodable.
#[derive(Encode)]
struct EncodeBasic {
    a: i8,
    b: u8,
    c: bool,
    d: i16,
    e: u16,
    f: i32,
    g: u32,
    h: i64,
    i: u64,
    j: f32,
    k: f64,
}

#[derive(Encode)]
struct EncodeWithOption {
    required: i32,
    optional: Option<i32>,
}

#[derive(Encode)]
struct EncodeWithSlice<'a> {
    data: &'a [u8],
    count: VarInt,
}

#[derive(Encode)]
struct EncodeWithSpecialTypes {
    varint: VarInt,
    varlong: VarLong,
    uuid: Uuid,
}

#[derive(Encode)]
struct EncodeUnit;

#[derive(Encode)]
struct EncodeTuple(i32);

#[derive(Encode)]
struct EncodeMultiTuple(i32, i64, bool);

#[derive(Encode)]
struct EncodeWithOwned<'a> {
    name: &'a str,
    data: Vec<u8>,
}

#[derive(Encode)]
struct EncodeWithString {
    owned: String,
}

#[derive(Encode)]
struct EncodeWithNbt<'a> {
    id: VarInt,
    data: NbtTag<'a>,
}

// `#[encode(rest)]` writes the raw bytes with no length prefix.
#[derive(Encode)]
struct EncodeWithRest<'a> {
    channel: &'a str,
    #[encode(rest)]
    payload: &'a [u8],
}

// `#[encode(len = "field")]` also writes raw bytes.
#[derive(Encode)]
struct EncodeWithLen<'a> {
    size: VarInt,
    #[encode(len = "size")]
    payload: &'a [u8],
}

// EncodePacket: unit, single-field and struct variants.
#[derive(EncodePacket)]
enum EncodePacketEnum {
    #[id(0)]
    Variant1(i32),
    #[id(1)]
    Variant2,
    #[id(2)]
    Variant3(VarInt),
}

#[derive(EncodePacket)]
enum EncodePacketStructVariant {
    #[id(10)]
    Simple(i32),
    #[id(11)]
    Complex { x: i32, y: i64 },
    #[id(12)]
    Empty,
}

// A type can derive both directions.
#[derive(Decode, Encode)]
struct BothDirections {
    id: VarInt,
    flag: bool,
}

fn main() {
    // If this compiles, the test passes
    println!("All derive macros compile successfully!");
}
