#![allow(dead_code)]

pub fn encode_base64(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(data)
}

pub fn decode_base64(data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    Ok(STANDARD.decode(data)?)
}
