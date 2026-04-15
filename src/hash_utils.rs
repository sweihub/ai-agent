use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

pub fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for c in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u64);
    }
    hash
}

/// djb2 string hash — fast non-cryptographic hash returning a signed 32-bit int.
/// Deterministic across runtimes. Use as a fallback when you need on-disk-stable
/// output (e.g. cache directory names that must survive runtime upgrades).
pub fn djb2_hash(str: &str) -> i32 {
    let mut hash: i32 = 0;
    for c in str.chars() {
        hash = ((hash << 5) - hash + c as i32) | 0;
    }
    hash
}

/// Hash arbitrary content for change detection using SHA-256.
/// Not crypto-safe but collision-resistant enough for diff detection.
pub fn hash_content(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Hash two strings without allocating a concatenated temp string.
/// Uses incremental SHA-256 update with null separator.
pub fn hash_pair(a: &str, b: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(a.as_bytes());
    hasher.update(b"\0");
    hasher.update(b.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn murmur_hash(key: &str, seed: u32) -> u32 {
    let mut hash = seed;
    let data = key.as_bytes();
    let len = data.len();
    let mut i = 0;

    while i + 4 <= len {
        let mut k = u32::from(data[i])
            | (u32::from(data[i + 1]) << 8)
            | (u32::from(data[i + 2]) << 16)
            | (u32::from(data[i + 3]) << 24);

        k = k.wrapping_mul(0x5bd1e995);
        k ^= k >> 24;
        k = k.wrapping_mul(0x5bd1e995);

        hash = hash.wrapping_mul(0x5bd1e995);
        hash ^= k;

        i += 4;
    }

    hash ^= (len as u32).wrapping_mul(0x5bd1e995);
    hash = hash.wrapping_mul(0x5bd1e995);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x5bd1e995);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0x5bd1e995);
    hash ^= hash >> 15;

    hash
}
