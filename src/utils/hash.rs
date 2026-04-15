// Source: /data/home/swei/claudecode/openclaudecode/src/utils/hash.ts
#![allow(dead_code)]

use sha2::{Digest, Sha256};

pub fn djb2_hash(s: &str) -> i32 {
    let mut hash: i32 = 0;
    for c in s.chars() {
        hash = ((hash << 5) - hash + c as i32) | 0;
    }
    hash
}

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn hash_pair(a: &str, b: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(a.as_bytes());
    hasher.update(b"\0");
    hasher.update(b.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_djb2_hash() {
        assert_eq!(djb2_hash("hello"), djb2_hash("hello"));
    }

    #[test]
    fn test_hash_content() {
        let h = hash_content("test content");
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_hash_pair() {
        let h1 = hash_pair("a", "b");
        let h2 = hash_pair("a", "b");
        assert_eq!(h1, h2);
    }
}
