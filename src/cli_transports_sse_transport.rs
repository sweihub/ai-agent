#![allow(dead_code)]

use crate::cli_transports_transport::Transport;

pub struct SseTransport {
    url: String,
}

impl SseTransport {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

impl Transport for SseTransport {
    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn send(&mut self, _data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }

    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
