#![allow(dead_code)]

pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

pub fn hex_decode(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(s)
}

pub fn base64_encode(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}

pub fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
}

pub fn url_encode(s: &str) -> String {
    urlencoding::encode(s).to_string()
}

pub fn url_decode(s: &str) -> Result<String, urlencoding::DecodeError> {
    urlencoding::decode(s).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex() {
        assert_eq!(hex_encode(b"hello"), "68656c6c6f");
        assert_eq!(hex_decode("68656c6c6f").unwrap(), b"hello");
    }

    #[test]
    fn test_base64() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
    }

    #[test]
    fn test_url() {
        assert_eq!(url_encode("hello world"), "hello%20world");
    }
}
