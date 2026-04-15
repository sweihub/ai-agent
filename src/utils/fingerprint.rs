// Source: /data/home/swei/claudecode/openclaudecode/src/utils/fingerprint.ts
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn fingerprint(data: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

pub fn fingerprint_file(path: &std::path::Path) -> Result<u64, String> {
    let content = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(hasher.finish())
}

pub fn fingerprint_bytes(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
