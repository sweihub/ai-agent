pub struct Semaphore {
    permits: usize,
    available: usize,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            permits,
            available: permits,
        }
    }

    pub fn acquire(&mut self) -> bool {
        if self.available > 0 {
            self.available -= 1;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self) {
        if self.available < self.permits {
            self.available += 1;
        }
    }

    pub fn available(&self) -> usize {
        self.available
    }

    pub fn is_available(&self) -> bool {
        self.available > 0
    }
}
