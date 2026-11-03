use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

pub struct UnsureThreshold {
    pub value: AtomicI32,
    pub exists: AtomicBool,
}

impl UnsureThreshold {
    pub fn new() -> Self {
        Self { value: AtomicI32::new(0), exists: AtomicBool::new(false) }
    }

    pub fn set(&self, to: i32) {
        self.exists.store(true, Ordering::Release);
        self.value.store(to, Ordering::Release);
    }

    pub fn get(&self) -> Option<i32> {
        if self.exists.load(Ordering::Acquire) {
            return Some(self.value.load(Ordering::Acquire))
        };
        None
    }

    pub fn reset(&self) {
        self.exists.store(false, Ordering::Release);
    }
}