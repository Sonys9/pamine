/// Encodes a signed VarInt (32-bit two's-complement, 7 bits per byte).
pub fn varint(number: i32) -> Vec<u8> {
    let mut array = Vec::with_capacity(5);
    let mut value = number as u32;
    loop {
        let byte = (value & 0b0111_1111) as u8;
        value >>= 7;
        if value != 0 {
            array.push(byte | 0b1000_0000);
        } else {
            array.push(byte);
            break;
        }
    }
    array
}

/// Encodes a signed VarLong (64-bit two's-complement, 7 bits per byte).
pub fn varlong(number: i64) -> Vec<u8> {
    let mut array = Vec::with_capacity(10);
    let mut value = number as u64;
    loop {
        let byte = (value & 0b0111_1111) as u8;
        value >>= 7;
        if value != 0 {
            array.push(byte | 0b1000_0000);
        } else {
            array.push(byte);
            break;
        }
    }
    array
}

/// Encodes a VarInt-prefixed UTF-8 string.
pub fn string(value: &str) -> Vec<u8> {
    let mut out = varint(value.len() as i32);
    out.extend_from_slice(value.as_bytes());
    out
}

/// Encodes a VarInt-prefixed byte array.
pub fn bytes(value: &[u8]) -> Vec<u8> {
    let mut out = varint(value.len() as i32);
    out.extend_from_slice(value);
    out
}

/// Wraps a packet body (id + data) in the uncompressed frame format.
pub fn packet_frame(body: &[u8]) -> Vec<u8> {
    let mut out = varint(body.len() as i32);
    out.extend_from_slice(body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;

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
}
