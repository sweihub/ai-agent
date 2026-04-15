#![allow(dead_code)]

use std::collections::HashMap;

pub struct TransportConfig {
    pub url: String,
    pub timeout_ms: u64,
}

pub trait Transport {
    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
    fn receive(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
