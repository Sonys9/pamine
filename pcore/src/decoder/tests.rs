//! Decoder tests: varints, bounds, cursor bookkeeping and string handling.

use super::{Decoder, VarInt};

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

    // i32::MIN == 80 80 80 80 08
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

#[test]
fn primitives_are_big_endian() {
    let mut d = Decoder::new(&[0x01, 0x00]);
    assert_eq!(d.u16().unwrap(), 256);

    let mut d = Decoder::new(&[0x00, 0x00, 0x01, 0x00]);
    assert_eq!(d.i32().unwrap(), 256);

    let mut d = Decoder::new(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]);
    assert_eq!(d.i64().unwrap(), 256);
}

#[test]
fn bool_reads_nonzero_as_true() {
    let mut d = Decoder::new(&[0x00, 0x01, 0x02]);
    assert!(!d.bool().unwrap());
    assert!(d.bool().unwrap());
    assert!(d.bool().unwrap());
}

#[test]
fn uuid_reads_sixteen_bytes() {
    let bytes = [0xABu8; 16];
    let mut d = Decoder::new(&bytes);
    assert_eq!(d.uuid().unwrap().get(), bytes);
    assert_eq!(d.remaining(), 0);
}

#[test]
fn varint_wrapper_roundtrip() {
    let v = VarInt::new(-42);
    assert_eq!(v.get(), -42);
    let mut v = v;
    v.edit(7);
    assert_eq!(v.get(), 7);
}
