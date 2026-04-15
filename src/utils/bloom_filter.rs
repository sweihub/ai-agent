pub struct BloomFilter {
    bits: Vec<bool>,
    size: usize,
    hash_count: usize,
}

impl BloomFilter {
    pub fn new(size: usize, hash_count: usize) -> Self {
        Self {
            bits: vec![false; size],
            size,
            hash_count,
        }
    }

    fn hash(&self, item: &str, seed: usize) -> usize {
        let mut hash: u64 = 5381;
        for byte in item.as_bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(*byte as u64);
        }
        ((hash.wrapping_add(seed as u64)) as usize) % self.size
    }

    pub fn insert(&mut self, item: &str) {
        for i in 0..self.hash_count {
            let idx = self.hash(item, i);
            self.bits[idx] = true;
        }
    }

    pub fn might_contain(&self, item: &str) -> bool {
        for i in 0..self.hash_count {
            let idx = self.hash(item, i);
            if !self.bits[idx] {
                return false;
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.bits.fill(false);
    }
}
