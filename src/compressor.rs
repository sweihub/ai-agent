#![allow(dead_code)]

pub fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}
