//! Read file in range utilities.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Read a specific range of lines from a file
pub fn read_file_in_range(
    path: &Path,
    start_line: usize,
    end_line: usize,
) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line_num = index + 1; // 1-indexed

        if line_num >= start_line && line_num <= end_line {
            lines.push(line?);
        }

        if line_num > end_line {
            break;
        }
    }

    Ok(lines.join("\n"))
}

/// Read a specific byte range from a file
pub fn read_bytes_in_range(path: &Path, start: u64, end: u64) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(start))?;

    let mut buffer = Vec::new();
    let mut reader = BufReader::new(file);
    let bytes_to_read = (end - start) as usize;

    buffer.resize(bytes_to_read, 0);
    reader.read_exact(&mut buffer)?;

    Ok(buffer)
}

/// Get line count of a file
pub fn get_line_count(path: &Path) -> Result<usize, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader.lines().count())
}
