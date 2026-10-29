//! Packet framing: length prefix, optional compression header, zlib payload.
//!
//! ```text
//! uncompressed: len:VarInt  body[len]
//! compressed:   len:VarInt  data_len:VarInt  body
//! ```
//!
//! `data_len == 0` means the body is stored uncompressed even though
//! compression is enabled.

use std::io::Read;

use flate2::read::ZlibDecoder;
use pcore::decoder::Decoder;

/// One framed packet, still pointing into the receive buffer.
pub struct Frame<'a> {
    /// Packet body, positioned at the opcode.
    pub body: &'a [u8],
    /// Whether `body` still needs zlib decompression.
    pub compressed: bool,
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
                consumed: total,
            }));
        }
        Ok(Some(Self {
            body: inner.get(),
            compressed: true,
            consumed: total,
        }))
    }
}

/// Decompresses `payload` into `out`, returning the number of bytes written.
pub fn inflate(payload: &[u8], out: &mut Vec<u8>) -> eyre::Result<usize> {
    let mut decoder = ZlibDecoder::new(payload);
    Ok(decoder.read_to_end(out)?)
}
