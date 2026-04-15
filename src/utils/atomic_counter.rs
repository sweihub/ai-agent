use std::sync::Arc;

pub struct AtomicCounter {
    value: Arc<std::sync::atomic::AtomicU64>,
}

impl AtomicCounter {
    pub fn new(initial: u64) -> Self {
        Self {
            value: Arc::new(std::sync::atomic::AtomicU64::new(initial)),
        }
    }

    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn decrement(&self) -> u64 {
        self.value.fetch_sub(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn get(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn reset(&self) -> u64 {
        self.value.swap(0, std::sync::atomic::Ordering::SeqCst)
    }
}

impl Clone for AtomicCounter {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
        }
    }
}
