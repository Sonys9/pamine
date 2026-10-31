//! Round-trip tests: `Encode` must produce exactly the bytes `Decode` reads.
//!
//! Each case encodes a value, decodes it back and asserts both that the value
//! matches and that no bytes are left over. A leftover byte means the encoder
//! wrote more than the decoder consumes (or vice versa), which is the class of
//! bug that desynchronises a packet stream.

use pcore::decoder::{Decoder, NbtTag, Slot, Uuid, VarInt, VarLong};
use pcore::encoder::Encoder;
use pcore::{Decode, Encode as EncodeTrait};
use pcore_gen::{DecodePacket, Encode, EncodePacket};

/// Encodes `value`, decodes it back and checks the result.
///
/// Only usable for types without a lifetime: a borrowed type would tie the
/// decoder's lifetime to the value's, which the borrow checker rejects. Those
/// cases are checked by hand below.
fn roundtrip<T>(value: &T)
where
    T: EncodeTrait + for<'a> Decode<'a> + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::new();
    value.encode(&mut encoder);
    let bytes = encoder.into_bytes();

    let mut decoder = Decoder::new(&bytes);
    let decoded = T::decode(&mut decoder).expect("decode failed");
    assert_eq!(&decoded, value, "value mismatch");
    assert_eq!(decoder.remaining(), 0, "trailing bytes after decode");
}

#[derive(Decode, Encode, PartialEq, Debug)]
struct Primitives {
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

#[derive(Decode, Encode, PartialEq, Debug)]
struct Specials {
    varint: VarInt,
    varlong: VarLong,
}

#[derive(Decode, Encode, PartialEq, Debug)]
struct WithOption {
    required: i32,
    present: Option<i32>,
    absent: Option<i32>,
}

#[derive(Decode, Encode, PartialEq, Debug)]
struct WithOwned {
    name: String,
    payload: Vec<u8>,
}

#[derive(Decode, Encode, PartialEq, Debug)]
struct Unit;

#[derive(Decode, Encode, PartialEq, Debug)]
struct Tuple(i32, bool);

#[derive(DecodePacket, EncodePacket, PartialEq, Debug)]
enum Packet {
    #[id(0x00)]
    Simple(i32),
    #[id(0x01)]
    Empty,
    #[id(0x02)]
    Struct { x: i32, y: i64 },
}

#[test]
fn primitives_roundtrip() {
    roundtrip(&Primitives {
        a: -128,
        b: 255,
        c: true,
        d: -32768,
        e: 65535,
        f: i32::MIN,
        g: u32::MAX,
        h: i64::MIN,
        i: u64::MAX,
        j: 3.5,
        k: -1.25,
    });
}

#[test]
fn specials_roundtrip() {
    roundtrip(&Specials {
        varint: VarInt::new(-1),
        varlong: VarLong::new(i64::MIN),
    });
}

#[test]
fn option_roundtrip() {
    roundtrip(&WithOption {
        required: 7,
        present: Some(-42),
        absent: None,
    });
}

#[test]
fn owned_roundtrip() {
    roundtrip(&WithOwned {
        name: "hello".to_owned(),
        payload: vec![1, 2, 3],
    });
}

#[test]
fn unit_and_tuple_roundtrip() {
    roundtrip(&Unit);
    roundtrip(&Tuple(-5, true));
}

#[test]
fn uuid_roundtrip() {
    #[derive(Decode, Encode, Debug)]
    struct WithUuid {
        uuid: Uuid,
    }

    let value = WithUuid {
        uuid: Uuid::new([0xAB; 16]),
    };
    let mut encoder = Encoder::new();
    value.encode(&mut encoder);
    let bytes = encoder.into_bytes();
    assert_eq!(bytes, vec![0xAB; 16]);

    let mut decoder = Decoder::new(&bytes);
    let decoded = WithUuid::decode(&mut decoder).unwrap();
    assert_eq!(decoded.uuid.get(), value.uuid.get());
    assert_eq!(decoder.remaining(), 0);
}

#[test]
fn borrowed_roundtrip() {
    #[derive(Decode, Encode, PartialEq, Debug)]
    struct WithBorrowed<'a> {
        name: &'a str,
        payload: &'a [u8],
    }

    let value = WithBorrowed {
        name: "hello",
        payload: &[1, 2, 3],
    };
    let mut encoder = Encoder::new();
    value.encode(&mut encoder);
    let bytes = encoder.into_bytes();

    let mut decoder = Decoder::new(&bytes);
    let decoded = WithBorrowed::decode(&mut decoder).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(decoder.remaining(), 0);
}

#[test]
fn nbt_roundtrip() {
    #[derive(Decode, Encode, PartialEq, Debug)]
    struct WithNbt<'a> {
        id: VarInt,
        data: NbtTag<'a>,
    }

    let value = WithNbt {
        id: VarInt::new(3),
        data: NbtTag::Compound(vec![
            ("a", NbtTag::Byte(1)),
            ("b", NbtTag::Int(256)),
            ("c", NbtTag::String("hi")),
            ("d", NbtTag::LongArray(vec![1, -2])),
            ("e", NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)])),
        ]),
    };
    let mut encoder = Encoder::new();
    value.encode(&mut encoder);
    let bytes = encoder.into_bytes();

    let mut decoder = Decoder::new(&bytes);
    let decoded = WithNbt::decode(&mut decoder).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(decoder.remaining(), 0);
}

#[test]
fn slot_roundtrip() {
    #[derive(Decode, Encode, PartialEq, Debug)]
    struct WithSlot<'a> {
        slot: Slot<'a>,
    }

    for slot in [
        Slot {
            present: true,
            item_id: Some(VarInt::new(5)),
            count: Some(3),
            nbt: Some(NbtTag::End),
        },
        Slot::empty(),
    ] {
        let value = WithSlot { slot };
        let mut encoder = Encoder::new();
        value.encode(&mut encoder);
        let bytes = encoder.into_bytes();

        let mut decoder = Decoder::new(&bytes);
        let decoded = WithSlot::decode(&mut decoder).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(decoder.remaining(), 0);
    }
}

#[test]
fn packet_enum_roundtrip() {
    for value in [
        Packet::Simple(-7),
        Packet::Empty,
        Packet::Struct { x: 1, y: -2 },
    ] {
        let mut encoder = Encoder::new();
        value.encode(&mut encoder);
        let bytes = encoder.into_bytes();

        // The opcode is the first VarInt; the body follows.
        let mut decoder = Decoder::new(&bytes);
        let opcode = decoder.varint().unwrap().get();
        let decoded = Packet::decode_by_opcode(opcode, &mut decoder).unwrap();

        assert_eq!(decoded, value);
        assert_eq!(decoder.remaining(), 0);
    }
}

#[test]
fn packet_enum_writes_opcode_first() {
    let mut encoder = Encoder::new();
    Packet::Simple(0).encode(&mut encoder);
    let bytes = encoder.into_bytes();
    assert_eq!(bytes[0], 0x00, "opcode must come first");

    let mut encoder = Encoder::new();
    Packet::Struct { x: 0, y: 0 }.encode(&mut encoder);
    let bytes = encoder.into_bytes();
    assert_eq!(bytes[0], 0x02, "opcode must come first");
}

#[test]
fn packet_enum_exposes_its_opcode() {
    // `opcode()` must agree with the `#[id(N)]` attribute for every variant
    // shape: tuple, unit and struct.
    assert_eq!(Packet::Simple(0).opcode(), 0x00);
    assert_eq!(Packet::Empty.opcode(), 0x01);
    assert_eq!(Packet::Struct { x: 0, y: 0 }.opcode(), 0x02);
}

#[test]
fn opcode_matches_the_first_encoded_varint() {
    // The whole point of `opcode()` is that it is the same value `Encode`
    // writes, so a caller can build a frame without hard-coding the id.
    for value in [
        Packet::Simple(-7),
        Packet::Empty,
        Packet::Struct { x: 1, y: -2 },
    ] {
        let mut encoder = Encoder::new();
        value.encode(&mut encoder);
        let bytes = encoder.into_bytes();

        let mut decoder = Decoder::new(&bytes);
        let written = decoder.varint().unwrap().get();
        assert_eq!(written, value.opcode(), "opcode() disagrees with Encode");
    }
}

#[test]
fn encode_matches_known_wire_bytes() {
    // A hand-checked fixture: VarInt 300, string "hi", bool true.
    #[derive(Encode)]
    struct Fixture<'a> {
        n: VarInt,
        s: &'a str,
        b: bool,
    }

    let mut encoder = Encoder::new();
    Fixture {
        n: VarInt::new(300),
        s: "hi",
        b: true,
    }
    .encode(&mut encoder);
    let bytes = encoder.into_bytes();

    assert_eq!(bytes, vec![0xAC, 0x02, 0x02, b'h', b'i', 0x01]);
}
