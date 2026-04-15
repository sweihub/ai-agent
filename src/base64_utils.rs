use std::io::Write;

pub fn encode_base64(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

pub fn decode_base64(encoded: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| e.to_string())
}

pub fn encode_base64_url(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE.encode(data)
}

pub fn decode_base64_url(encoded: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE
        .decode(encoded)
        .map_err(|e| e.to_string())
}

pub fn encode_base64_file(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(data)
}

pub fn decode_base64_file(encoded: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(encoded)
        .map_err(|e| e.to_string())
}
