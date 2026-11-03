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

    /// Returns the bytes of the packet that was just parsed.
    ///
    /// The caller clears the buffer after forwarding a packet, so `diff` is the
    /// size of exactly one packet here.
    pub fn packet_bytes(&self) -> &[u8] {
        &self.buffer.buffer[..self.diff]
    }

    /// Appends freshly received bytes for one direction.
    ///
    /// [`BigBuffer::update`] drops the oldest bytes when the buffer would
    /// overflow, so the read cursor has to be shifted by the same amount.
    /// Dropping *unparsed* bytes would silently corrupt the stream, so that is
    /// reported as an error instead.
    ///
    /// This returns a `Result` rather than panicking: the caller runs inside a
    /// spawned task, where a panic aborts the task and leaves the peer's socket
    /// open. A hostile peer could otherwise kill the task with a single
    /// oversized write.
    pub fn update(&mut self, bytes: &[u8]) -> eyre::Result<()> {
        let before = self.buffer.len;
        let start = self.buffer.update(bytes, bytes.len());
        let dropped = before - start;
        if dropped > self.diff {
            eyre::bail!(
                "receive buffer overflow: dropped {} unparsed bytes",
                dropped - self.diff
            );
        }
        self.diff -= dropped;
        Ok(())
    }
}
