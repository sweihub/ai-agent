#![allow(dead_code)]

const BASE_58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const VERSION: &str = "01";
const ENCODED_LENGTH: usize = 22;

fn base58_encode(n: u128) -> String {
    let base = BASE_58_CHARS.len() as u128;
    let mut result = vec![BASE_58_CHARS.chars().next().unwrap(); ENCODED_LENGTH];
    let mut i = ENCODED_LENGTH - 1;
    let mut value = n;

    while value > 0 {
        let rem = (value % base) as usize;
        result[i] = BASE_58_CHARS.chars().nth(rem).unwrap();
        value = value / base;
        i -= 1;
    }

    result.into_iter().collect()
}

fn uuid_to_bigint(uuid: &str) -> u128 {
    let hex = uuid.replace('-', "");
    if hex.len() != 32 {
        panic!("Invalid UUID hex length: {}", hex.len());
    }
    u128::from_str_radix(&hex, 16).expect("Invalid hex")
}

pub fn to_tagged_id(tag: &str, uuid: &str) -> String {
    let n = uuid_to_bigint(uuid);
    format!("{}_{}{}", tag, VERSION, base58_encode(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagged_id() {
        let result = to_tagged_id("user", "550e8400-e29b-41d4-a716-446655440000");
        assert!(result.starts_with("user_01"));
    }
}
