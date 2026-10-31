//! Packet framing: the uncompressed length prefix.
//!
//! ```text
//! uncompressed: len:VarInt  body[len]
//! compressed:   len:VarInt  data_len:VarInt  body
//! ```
//!
//! Only the uncompressed form is produced here; compression is applied by the
//! caller, which knows the threshold.

use super::varint::write_varint;

/// Wraps a packet body (id + data) in the uncompressed frame format.
pub fn packet_frame(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 5);
    write_varint(&mut out, body.len() as i32);
    out.extend_from_slice(body);
    out
}
