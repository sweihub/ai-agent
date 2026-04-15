//! Configuration types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub values: HashMap<String, serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl Config {
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.values.insert(key, value);
    }
}
