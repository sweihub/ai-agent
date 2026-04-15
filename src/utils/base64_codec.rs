pub struct Base64Codec;

impl Base64Codec {
    pub fn encode(data: &[u8]) -> String {
        use std::io::Write;
        let mut encoder =
            base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
        encoder.write_all(data).unwrap();
        encoder.into_inner()
    }

    pub fn decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
        use std::io::Read;
        let mut decoder = base64::read::DecoderStringReader::new(
            &base64::engine::general_purpose::STANDARD,
            input,
        );
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        Ok(buf)
    }
}
