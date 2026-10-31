//! Packet framing: length prefix, optional compression header, zlib payload.
//!
//! ```text
//! uncompressed: len:VarInt  body[len]
//! compressed:   len:VarInt  data_len:VarInt  body
//! ```
//!
//! `data_len == 0` means the body is stored uncompressed even though
//! compression is enabled.

use std::io::{Read, Write};

use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use pcore::{
    decoder::Decoder,
    encoder::{packet_frame, write_varint},
};

/// Largest packet body the protocol allows, compressed or not.
///
/// Vanilla 1.19.2 never sends a body above 2 MiB, and the receive buffer is
/// sized to match. Anything larger is either a bug or a zlib bomb, so it is
/// rejected before a single byte is decompressed.
pub const MAX_PACKET_LEN: usize = 2 * 1024 * 1024;

/// One framed packet, still pointing into the receive buffer.
pub struct Frame<'a> {
    /// Packet body, positioned at the opcode.
    pub body: &'a [u8],
    /// Whether `body` still needs zlib decompression.
    pub compressed: bool,
    /// Uncompressed size declared by the `data_len` header, or `0` when the
    /// frame is not compressed.
    ///
    /// This is the size the peer promises the body will have after inflating.
    /// It is validated against [`MAX_PACKET_LEN`] in [`Frame::split`] and
    /// against the actual output in [`inflate`], so a lying header cannot make
    /// the decompressor allocate more than it declared.
    pub data_len: usize,
    /// Total bytes this frame occupies in the receive buffer, length prefix
    /// included. This is what the read cursor must advance by.
    pub consumed: usize,
}

impl<'a> Frame<'a> {
    /// Splits the first frame off `pending`.
    ///
    /// Returns `Ok(None)` when `pending` does not yet contain a whole frame.
    pub fn split(pending: &'a [u8], threshold: Option<i32>) -> eyre::Result<Option<Self>> {
        let mut head = Decoder::new(pending);
        let Ok(len) = head.varint().map(|v| v.get()) else {
            // Not even the length prefix is complete yet.
            return Ok(None);
        };
        if len < 0 {
            eyre::bail!("negative packet length: {len}");
        }
        let len = len as usize;
        // Reject an oversized frame before waiting for its bytes: otherwise a
        // peer could make us buffer up to `len` bytes just to fail later.
        if len > MAX_PACKET_LEN {
            eyre::bail!("packet length {len} exceeds the {MAX_PACKET_LEN} byte limit");
        }
        let prefix_len = head.len();
        let total = len
            .checked_add(prefix_len)
            .ok_or_else(|| eyre::eyre!("packet length overflow"))?;

        if pending.len() < total {
            return Ok(None);
        }

        let body = &pending[prefix_len..total];

        let Some(_threshold) = threshold else {
            return Ok(Some(Self {
                body,
                compressed: false,
                data_len: 0,
                consumed: total,
            }));
        };

        // Compression is on: the body starts with the uncompressed length.
        let mut inner = Decoder::new(body);
        let data_len = inner.varint()?.get();
        if data_len <= 0 {
            // Stored uncompressed despite compression being enabled.
            return Ok(Some(Self {
                body: inner.get(),
                compressed: false,
                data_len: 0,
                consumed: total,
            }));
        }
        let data_len = data_len as usize;
        // The declared size is what the decompressor will be allowed to write,
        // so it has to be bounded before `inflate` runs. A zlib stream can
        // expand ~1000:1, so a 2 MiB frame could otherwise claim gigabytes.
        if data_len > MAX_PACKET_LEN {
            eyre::bail!("declared data_len {data_len} exceeds the {MAX_PACKET_LEN} byte limit");
        }
        Ok(Some(Self {
            body: inner.get(),
            compressed: true,
            data_len,
            consumed: total,
        }))
    }
}

/// Decompresses `payload` into `out`, returning the number of bytes written.
///
/// `expected` is the `data_len` the peer declared. The reader is capped at
/// `expected + 1` bytes, so a stream that expands past its own header is
/// stopped immediately instead of filling memory, and the size is then checked
/// for an exact match. A mismatch means the header lied, which is a protocol
/// violation rather than something to paper over.
pub fn inflate(payload: &[u8], out: &mut Vec<u8>, expected: usize) -> eyre::Result<usize> {
    let mut decoder = ZlibDecoder::new(payload).take(expected as u64 + 1);
    let written = decoder.read_to_end(out)?;
    if written != expected {
        eyre::bail!("decompressed {written} bytes, but data_len declared {expected}");
    }
    Ok(written)
}

/// Compresses `payload` with zlib.
pub fn deflate(payload: &[u8]) -> eyre::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(payload)?;
    Ok(encoder.finish()?)
}

/// Wraps a packet body (opcode + fields) into a frame, honouring compression.
///
/// This is the write-side counterpart of [`Frame::split`]. `threshold` is the
/// value the server sent in `Set Compression`:
///
/// - `None` — the connection is uncompressed, so the frame is just
///   `len:VarInt body`.
/// - `Some(n)` — every frame carries a `data_len:VarInt` header. Bodies of at
///   least `n` bytes are zlib-compressed and `data_len` is their uncompressed
///   size; shorter bodies are stored as-is with `data_len = 0`.
///
/// A packet written without this header on a compressed connection is read by
/// the peer as a bogus `data_len` and the connection is dropped.
pub fn frame_packet(body: &[u8], threshold: Option<i32>) -> eyre::Result<Vec<u8>> {
    let Some(threshold) = threshold else {
        return Ok(packet_frame(body));
    };

    // `data_len` is the uncompressed size, or 0 when the body is stored raw.
    let mut inner = Vec::with_capacity(body.len() + 5);
    if body.len() >= threshold.max(0) as usize {
        let compressed = deflate(body)?;
        write_varint(&mut inner, body.len() as i32);
        inner.extend_from_slice(&compressed);
    } else {
        write_varint(&mut inner, 0);
        inner.extend_from_slice(body);
    }

    let mut out = Vec::with_capacity(inner.len() + 5);
    write_varint(&mut out, inner.len() as i32);
    out.extend_from_slice(&inner);
    Ok(out)
}
