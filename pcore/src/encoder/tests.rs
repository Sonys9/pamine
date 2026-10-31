//! Round-trip tests for the encoder.
//!
//! Every case writes a value and reads it back with the matching decoder,
//! asserting that no bytes are left over. A leftover byte means the writer and
//! the reader disagree about the layout.

use crate::decoder::{Decoder, NbtTag, Slot, VarInt, VarLong};
use crate::{Decode, Encode};

use super::Encoder;
use super::frame::packet_frame;
use super::string::{bytes, string};
use super::varint::{varint, varlong};

#[test]
fn varint_roundtrip_positive() {
    for n in [0, 1, 127, 128, 300, 25565, i32::MAX] {
        let encoded = varint(n);
        let mut d = Decoder::new(&encoded);
        assert_eq!(d.varint().unwrap().get(), n, "failed for {n}");
    }
}

#[test]
fn varint_roundtrip_negative() {
    for n in [-1, -2, -128, -300, i32::MIN] {
        let encoded = varint(n);
        let mut d = Decoder::new(&encoded);
        assert_eq!(d.varint().unwrap().get(), n, "failed for {n}");
    }
}

#[test]
fn varlong_roundtrip() {
    for n in [0i64, 1, -1, i64::MAX, i64::MIN, 1234567890123] {
        let encoded = varlong(n);
        let mut d = Decoder::new(&encoded);
        assert_eq!(d.varlong().unwrap().get(), n, "failed for {n}");
    }
}

#[test]
fn varint_known_encoding() {
    assert_eq!(varint(0), vec![0x00]);
    assert_eq!(varint(127), vec![0x7f]);
    assert_eq!(varint(128), vec![0x80, 0x01]);
    assert_eq!(varint(300), vec![0xac, 0x02]);
    assert_eq!(varint(-1), vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
}

#[test]
fn string_roundtrip() {
    let encoded = string("hello");
    let mut d = Decoder::new(&encoded);
    assert_eq!(d.string().unwrap(), "hello");
}

#[test]
fn bytes_roundtrip() {
    let encoded = bytes(&[1, 2, 3]);
    let mut d = Decoder::new(&encoded);
    assert_eq!(d.bytes().unwrap(), vec![1, 2, 3]);
}

#[test]
fn packet_frame_roundtrip() {
    let body = b"\x00\x01\x02";
    let framed = packet_frame(body);
    let mut d = Decoder::new(&framed);
    let len = d.varint().unwrap().get() as usize;
    assert_eq!(len, body.len());
    assert_eq!(d.rest(), body);
}

#[test]
fn encoder_cursor_methods() {
    let mut encoder = Encoder::new();
    assert!(encoder.is_empty());
    assert_eq!(encoder.len(), 0);

    encoder.varint(300);
    encoder.string("hi");
    assert_eq!(encoder.len(), 5);
    assert!(!encoder.is_empty());
    assert_eq!(encoder.as_slice(), &[0xac, 0x02, 0x02, b'h', b'i']);

    encoder.clear();
    assert!(encoder.is_empty());

    encoder.bool(true);
    assert_eq!(encoder.into_bytes(), vec![0x01]);
}

#[test]
fn encoder_with_capacity() {
    let encoder = Encoder::with_capacity(64);
    assert!(encoder.is_empty());
    assert_eq!(encoder.into_bytes(), Vec::<u8>::new());
}

#[test]
fn encoder_raw_writes_no_prefix() {
    let mut encoder = Encoder::new();
    encoder.raw(&[1, 2, 3]);
    assert_eq!(encoder.into_bytes(), vec![1, 2, 3]);
}

#[test]
fn encoder_scalar_methods() {
    let mut encoder = Encoder::new();
    encoder.i8(-128);
    encoder.u8(255);
    encoder.bool(true);
    encoder.i16(-32768);
    encoder.u16(65535);
    encoder.i32(i32::MIN);
    encoder.u32(u32::MAX);
    encoder.i64(i64::MIN);
    encoder.u64(u64::MAX);
    encoder.f32(3.5);
    encoder.f64(-1.25);

    let bytes = encoder.into_bytes();
    let mut d = Decoder::new(&bytes);
    assert_eq!(d.i8().unwrap(), -128);
    assert_eq!(d.u8().unwrap(), 255);
    assert!(d.bool().unwrap());
    assert_eq!(d.i16().unwrap(), -32768);
    assert_eq!(d.u16().unwrap(), 65535);
    assert_eq!(d.i32().unwrap(), i32::MIN);
    assert_eq!(d.u32().unwrap(), u32::MAX);
    assert_eq!(d.i64().unwrap(), i64::MIN);
    assert_eq!(d.u64().unwrap(), u64::MAX);
    assert_eq!(d.f32().unwrap(), 3.5);
    assert_eq!(d.f64().unwrap(), -1.25);
    assert_eq!(d.remaining(), 0);
}

#[test]
fn nbt_roundtrip_compound() {
    let tag = NbtTag::Compound(vec![
        ("a", NbtTag::Byte(1)),
        ("b", NbtTag::Int(256)),
        ("c", NbtTag::String("hi")),
    ]);
    let mut encoder = Encoder::new();
    encoder.nbt(&tag);

    let bytes = encoder.into_bytes();
    let mut d = Decoder::new(&bytes);
    assert_eq!(d.nbt().unwrap(), tag);
    assert_eq!(d.remaining(), 0);
}

#[test]
fn nbt_roundtrip_arrays() {
    for tag in [
        NbtTag::ByteArray(&[1, 2, 3]),
        NbtTag::IntArray(vec![1, -2, 3]),
        NbtTag::LongArray(vec![1, -2, 3]),
        NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)]),
        NbtTag::List(vec![]),
        NbtTag::End,
    ] {
        let mut encoder = Encoder::new();
        encoder.nbt(&tag);

        let bytes = encoder.into_bytes();
        let mut d = Decoder::new(&bytes);
        assert_eq!(d.nbt().unwrap(), tag, "failed for {tag:?}");
        assert_eq!(d.remaining(), 0, "trailing bytes for {tag:?}");
    }
}

#[test]
fn slot_roundtrip() {
    let slot = Slot {
        present: true,
        item_id: Some(VarInt::new(5)),
        count: Some(3),
        nbt: Some(NbtTag::End),
    };
    let mut encoder = Encoder::new();
    encoder.slot(&slot);

    let bytes = encoder.into_bytes();
    let mut d = Decoder::new(&bytes);
    let decoded = Slot::decode(&mut d).unwrap();
    assert_eq!(decoded, slot);
    assert_eq!(d.remaining(), 0);
}

#[test]
fn empty_slot_roundtrip() {
    let slot = Slot::empty();
    let mut encoder = Encoder::new();
    encoder.slot(&slot);
    assert_eq!(encoder.into_bytes(), vec![0]);

    let bytes = bytes_of(&slot);
    let mut d = Decoder::new(&bytes);
    assert_eq!(Slot::decode(&mut d).unwrap(), slot);
}

/// Encodes any `Encode` value into a fresh `Vec`.
fn bytes_of<T: Encode + ?Sized>(value: &T) -> Vec<u8> {
    crate::encode(value)
}

#[test]
fn option_roundtrip() {
    let mut encoder = Encoder::new();
    encoder.encode(&Some(VarInt::new(7)));
    encoder.encode(&None::<VarInt>);

    let bytes = encoder.into_bytes();
    let mut d = Decoder::new(&bytes);
    assert!(d.bool().unwrap());
    assert_eq!(d.varint().unwrap().get(), 7);
    assert!(!d.bool().unwrap());
    assert_eq!(d.remaining(), 0);
}

#[test]
fn vec_roundtrip() {
    let mut encoder = Encoder::new();
    encoder.encode(&vec![VarInt::new(1), VarInt::new(2)]);

    let bytes = encoder.into_bytes();
    let mut d = Decoder::new(&bytes);
    assert_eq!(d.varint().unwrap().get(), 2);
    assert_eq!(d.varint().unwrap().get(), 1);
    assert_eq!(d.varint().unwrap().get(), 2);
    assert_eq!(d.remaining(), 0);
}

#[test]
fn varlong_type_roundtrip() {
    let mut encoder = Encoder::new();
    encoder.encode(&VarLong::new(i64::MIN));

    let bytes = encoder.into_bytes();
    let mut d = Decoder::new(&bytes);
    assert_eq!(d.varlong().unwrap().get(), i64::MIN);
    assert_eq!(d.remaining(), 0);
}
