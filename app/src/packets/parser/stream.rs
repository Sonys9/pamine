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
}
