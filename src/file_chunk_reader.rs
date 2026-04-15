#![allow(dead_code)]

pub fn read_file_chunk(path: &str, offset: usize, length: usize) -> Result<String, std::io::Error> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset as u64))?;
    let mut buffer = vec![0u8; length];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    Ok(String::from_utf8_lossy(&buffer).to_string())
}
