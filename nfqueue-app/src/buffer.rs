pub const BUFFER_LEN: usize = 1024 * 1024 * 2 + 1024 * 64; // 2 MB + 64 KB

pub struct BigBuffer {
    pub buffer: Vec<u8>,
    pub len: usize,
}

impl Default for BigBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl BigBuffer {
    pub fn new() -> Self {
        Self {
            buffer: vec![0u8; BUFFER_LEN],
            len: 0,
        }
    }

    /// Appends `len` bytes from `buffer` and returns the offset they were written at.
    ///
    /// If the live region would overflow, the oldest bytes are dropped first, so the
    /// returned offset is relative to the *post-drop* buffer contents.
    pub fn update(&mut self, buffer: &[u8], len: usize) -> usize {
        if len > BUFFER_LEN {
            panic!("got too big packet: {len} > {BUFFER_LEN}");
        }
        if buffer.len() < len {
            panic!("source slice is shorter than len: {} < {len}", buffer.len());
        }
        let clean_buffer = &buffer[..len];

        // Drop the oldest bytes so that `len` new ones fit.
        if self.len + len > BUFFER_LEN {
            let drop = self.len + len - BUFFER_LEN;
            self.move_buffer(drop, 0);
        }

        let start = self.len;
        self.buffer[start..start + len].copy_from_slice(clean_buffer);
        self.len += len;
        start
    }

    #[inline]
    pub fn remove(&mut self, start: usize, end: usize) {
        if start > end {
            panic!("Can't remove due to start > end!");
        };
        self.move_buffer(end, start);
    }

    /// Shifts the live region `[index..len]` down to `dest`.
    ///
    /// `index` is the absolute offset of the first byte to keep, `dest` is where
    /// it should land. Both must be within the live region, otherwise the
    /// subtraction below would underflow.
    #[inline]
    pub fn move_buffer(&mut self, index: usize, dest: usize) {
        if dest > index {
            panic!("Can't move buffer forward: dest {dest} > index {index}");
        }
        if index > self.len {
            panic!("Can't move buffer: index {index} > len {}", self.len);
        }
        self.buffer.copy_within(index..self.len, dest);
        self.len -= index - dest;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_returns_start_offset() {
        let mut b = BigBuffer::new();
        assert_eq!(b.update(b"abc", 3), 0);
        assert_eq!(b.update(b"de", 2), 3);
        assert_eq!(b.len, 5);
        assert_eq!(&b.buffer[..5], b"abcde");
    }

    #[test]
    fn append_respects_len_smaller_than_slice() {
        let mut b = BigBuffer::new();
        b.update(b"abcdef", 3);
        assert_eq!(b.len, 3);
        assert_eq!(&b.buffer[..3], b"abc");
    }

    #[test]
    fn move_buffer_shifts_and_shrinks() {
        let mut b = BigBuffer::new();
        b.update(b"abcdef", 6);
        b.move_buffer(2, 0);
        assert_eq!(b.len, 4);
        assert_eq!(&b.buffer[..4], b"cdef");
    }

    #[test]
    fn move_buffer_noop_when_index_equals_dest() {
        let mut b = BigBuffer::new();
        b.update(b"abc", 3);
        b.move_buffer(0, 0);
        assert_eq!(b.len, 3);
        assert_eq!(&b.buffer[..3], b"abc");
    }

    #[test]
    fn remove_drops_range() {
        let mut b = BigBuffer::new();
        b.update(b"abcdef", 6);
        b.remove(1, 3); // drop "bc"
        assert_eq!(b.len, 4);
        assert_eq!(&b.buffer[..4], b"adef");
    }

    #[test]
    fn overflow_drops_oldest_bytes() {
        let mut b = BigBuffer::new();
        // Fill exactly to capacity.
        let chunk = vec![1u8; BUFFER_LEN];
        b.update(&chunk, BUFFER_LEN);
        assert_eq!(b.len, BUFFER_LEN);

        // Two more bytes must evict the two oldest.
        let start = b.update(b"XY", 2);
        assert_eq!(b.len, BUFFER_LEN);
        assert_eq!(start, BUFFER_LEN - 2);
        assert_eq!(&b.buffer[BUFFER_LEN - 2..], b"XY");
        // The very first byte is now the third original byte (still 1).
        assert_eq!(b.buffer[0], 1);
    }

    #[test]
    fn overflow_with_large_chunk_keeps_tail() {
        let mut b = BigBuffer::new();
        b.update(b"old", 3);
        let chunk = vec![7u8; BUFFER_LEN];
        let start = b.update(&chunk, BUFFER_LEN);
        assert_eq!(b.len, BUFFER_LEN);
        assert_eq!(start, 0);
        assert!(b.buffer[..BUFFER_LEN].iter().all(|&x| x == 7));
    }

    #[test]
    #[should_panic(expected = "got too big packet")]
    fn rejects_oversized_chunk() {
        let mut b = BigBuffer::new();
        let chunk = vec![0u8; BUFFER_LEN + 1];
        b.update(&chunk, BUFFER_LEN + 1);
    }

    #[test]
    #[should_panic(expected = "source slice is shorter than len")]
    fn rejects_len_beyond_slice() {
        let mut b = BigBuffer::new();
        b.update(b"ab", 5);
    }

    #[test]
    #[should_panic(expected = "Can't move buffer forward")]
    fn move_buffer_rejects_forward_move() {
        let mut b = BigBuffer::new();
        b.update(b"abc", 3);
        b.move_buffer(0, 2);
    }

    #[test]
    #[should_panic(expected = "Can't move buffer")]
    fn move_buffer_rejects_index_past_len() {
        let mut b = BigBuffer::new();
        b.update(b"abc", 3);
        b.move_buffer(10, 0);
    }
}
