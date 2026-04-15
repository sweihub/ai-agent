pub struct BitSet {
    bits: Vec<u64>,
    size: usize,
}

impl BitSet {
    pub fn new(size: usize) -> Self {
        let words = (size + 63) / 64;
        Self {
            bits: vec![0; words],
            size,
        }
    }

    pub fn set(&mut self, index: usize) {
        if index < self.size {
            let word = index / 64;
            let bit = index % 64;
            self.bits[word] |= 1 << bit;
        }
    }

    pub fn clear(&mut self, index: usize) {
        if index < self.size {
            let word = index / 64;
            let bit = index % 64;
            self.bits[word] &= !(1 << bit);
        }
    }

    pub fn test(&self, index: usize) -> bool {
        if index < self.size {
            let word = index / 64;
            let bit = index % 64;
            (self.bits[word] & (1 << bit)) != 0
        } else {
            false
        }
    }

    pub fn count(&self) -> usize {
        self.bits.iter().map(|w| w.count_ones() as usize).sum()
    }
}
