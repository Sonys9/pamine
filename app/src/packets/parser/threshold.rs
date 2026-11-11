use std::sync::{Arc, atomic::{AtomicBool, AtomicI32, Ordering}};

/// The compression threshold, which is `None` until `Set Compression` arrives.
///
/// Two atomics are used instead of a `Mutex` because the value is read on every
/// packet in both directions. The store order matters: `value` is written
/// *before* `exists`, so a reader that observes `exists == true` is guaranteed
/// to see the matching `value`. Storing `exists` first would let a reader see
/// `true` with a stale `value` of `0`, which would make it treat an
/// uncompressed packet as compressed and desynchronise the stream.
/// 
/// Arc<..> is for Clone
#[derive(Debug, Clone)]
pub struct UnsureThreshold {
    pub value: Arc<AtomicI32>,
    pub exists: Arc<AtomicBool>,
}

impl Default for UnsureThreshold {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsureThreshold {
    pub fn new() -> Self {
        Self {
            value: Arc::new(AtomicI32::new(0)),
            exists: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set(&self, to: i32) {
        // Value first, flag second: see the type-level comment.
        self.value.store(to, Ordering::Release);
        self.exists.store(true, Ordering::Release);
    }

    pub fn get(&self) -> Option<i32> {
        if self.exists.load(Ordering::Acquire) {
            Some(self.value.load(Ordering::Acquire))
        } else {
            None
        }
    }

    pub fn reset(&self) {
        self.exists.store(false, Ordering::Release);
    }
}
