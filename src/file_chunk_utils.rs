use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

pub struct FileChunkReader {
    path: String,
    chunk_size: usize,
    current_line: usize,
}

impl FileChunkReader {
    pub fn new(path: &str, chunk_size: usize) -> Self {
        FileChunkReader {
            path: path.to_string(),
            chunk_size,
            current_line: 0,
        }
    }

    pub fn next_chunk(&mut self) -> Result<Option<String>, String> {
        let file = File::open(&self.path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            if idx < self.current_line {
                continue;
            }
            if idx >= self.current_line + self.chunk_size {
                break;
            }
            lines.push(line.map_err(|e| e.to_string())?);
        }

        if lines.is_empty() {
            return Ok(None);
        }

        self.current_line += lines.len();
        Ok(Some(lines.join("\n")))
    }

    pub fn reset(&mut self) {
        self.current_line = 0;
    }

    pub fn current_position(&self) -> usize {
        self.current_line
    }
}

pub fn read_file_lines(path: &str) -> Result<Vec<String>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line.map_err(|e| e.to_string())?);
    }
    Ok(lines)
}

pub fn read_file_bytes(path: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    Ok(buffer)
}

pub fn get_file_size(path: &str) -> Result<u64, String> {
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(metadata.len())
}

pub fn count_lines(path: &str) -> Result<usize, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}
