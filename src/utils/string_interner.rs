pub struct StringInterner {
    strings: Vec<String>,
    map: std::collections::HashMap<String, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            map: std::collections::HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.map.get(s) {
            return idx;
        }
        let idx = self.strings.len();
        self.strings.push(s.to_string());
        self.map.insert(s.to_string(), idx);
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
