//! Per-direction receive buffer and its read cursor.

use crate::buffer::BigBuffer;

/// Receive buffer for one direction of the connection.
pub struct StreamData {
    pub buffer: BigBuffer,
    /// Offset of the first unparsed byte.
    pub(crate) diff: usize,
}

impl Default for StreamData {
    fn default() -> Self {
        Self {
            buffer: BigBuffer::new(),
            diff: 0,
        }
    }
}

impl StreamData {
    /// Bytes received but not yet parsed.
    pub fn pending(&self) -> &[u8] {
        &self.buffer.buffer[self.diff..self.buffer.len]
    }

    /// True when every received byte has been parsed.
    pub fn is_drained(&self) -> bool {
        self.diff >= self.buffer.len
    }

    /// Advances the read cursor by `n` bytes.
    pub fn advance(&mut self, n: usize) {
        self.diff += n;
    }

    /// Drops parsed bytes and rewinds the cursor.
    pub fn compact(&mut self) {
        self.buffer.move_buffer(self.diff, 0);
        self.diff = 0;
    }

    /// Drops already-parsed bytes and rewinds the read cursor.
    pub fn clear(&mut self) {
        self.compact();
    }

    /// True when no unparsed bytes remain for this direction.
    ///
    /// Note this is not the same as "the buffer is empty": parsed bytes stay in
    /// the buffer until [`clear`](Self::clear) compacts it.
    pub fn is_empty(&self) -> bool {
        self.is_drained()
    }

    /// Returns parsed packet's bytes
    pub fn packet_bytes(&mut self) -> &[u8] {
        &self.buffer.buffer[..self.diff]
    }

    /// Appends freshly received bytes for one direction.
    pub fn update(&mut self, bytes: &[u8]) {
        self.buffer.update(bytes, bytes.len());
    }
}
