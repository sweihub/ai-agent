//! Secure storage utilities.

use std::collections::HashMap;

/// Secure storage for sensitive data
pub struct SecureStorage {
    data: HashMap<String, Vec<u8>>,
}

impl SecureStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Store a value
    pub fn set(&mut self, key: &str, value: Vec<u8>) {
        self.data.insert(key.to_string(), value);
    }

    /// Retrieve a value
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    /// Check if a key exists
    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Delete a value
    pub fn delete(&mut self, key: &str) -> Option<Vec<u8>> {
        self.data.remove(key)
    }

    /// Clear all values
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for SecureStorage {
    fn default() -> Self {
        Self::new()
    }
}
