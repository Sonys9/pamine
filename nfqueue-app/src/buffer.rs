const BUFFER_LEN: usize = 1024 * 32;

pub struct BigBuffer {
    pub buffer: Vec<u8>,
    pub len: usize,
}

impl BigBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(BUFFER_LEN),
            len: 0,
        }
    }

    pub fn update(&mut self, buffer: &[u8], len: usize) -> usize {
        let clean_buffer = &buffer[..len];
        if BUFFER_LEN - self.len < len {
            let diff = self.len + len - 2048;
            let old_len = self.len - diff;
            self.move_buffer(diff, 0);
            self.buffer[old_len..old_len + len].copy_from_slice(clean_buffer);
            self.len = BUFFER_LEN;
            old_len
        } else {
            let sum = self.len + len;
            self.buffer[self.len..sum].copy_from_slice(clean_buffer);
            self.len = sum;
            self.len - len
        }
    }

    #[inline]
    pub fn remove(&mut self, start: usize, end: usize) {
        if start > end {
            panic!("Can't remove due to start > end!");
        };
        self.len = self.len - (end - start);
        self.move_buffer(end, start);
    }

    #[inline]
    pub fn move_buffer(&mut self, index: usize, dest: usize) {
        self.buffer.copy_within(index.., dest);
        self.len -= index - dest;
    }
}
