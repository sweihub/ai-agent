use std::io::Write;

pub fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = Vec::new();
    let mut gzip_encoder =
        flate2::write::GzEncoder::new(&mut encoder, flate2::Compression::default());
    gzip_encoder.write_all(data).map_err(|e| e.to_string())?;
    gzip_encoder.finish().map_err(|e| e.to_string())?;
    Ok(encoder)
}

pub fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut result = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut result).map_err(|e| e.to_string())?;
    Ok(result)
}

pub fn compress_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = Vec::new();
    let mut zlib_encoder =
        flate2::write::ZlibEncoder::new(&mut encoder, flate2::Compression::default());
    zlib_encoder.write_all(data).map_err(|e| e.to_string())?;
    zlib_encoder.finish().map_err(|e| e.to_string())?;
    Ok(encoder)
}

pub fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut result = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut result).map_err(|e| e.to_string())?;
    Ok(result)
}
