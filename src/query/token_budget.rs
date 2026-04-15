pub struct QueryTokenBudget {
    pub max_tokens: u64,
    pub used_tokens: u64,
    pub reserved_tokens: u64,
}

impl QueryTokenBudget {
    pub fn new(max_tokens: u64) -> Self {
        Self {
            max_tokens,
            used_tokens: 0,
            reserved_tokens: 0,
        }
    }

    pub fn allocate(&mut self, tokens: u64) -> bool {
        let available = self.available();
        if tokens <= available {
            self.used_tokens += tokens;
            true
        } else {
            false
        }
    }

    pub fn reserve(&mut self, tokens: u64) -> bool {
        let available = self.available();
        if tokens <= available {
            self.reserved_tokens += tokens;
            true
        } else {
            false
        }
    }

    pub fn available(&self) -> u64 {
        self.max_tokens
            .saturating_sub(self.used_tokens)
            .saturating_sub(self.reserved_tokens)
    }

    pub fn is_exhausted(&self) -> bool {
        self.available() == 0
    }
}
