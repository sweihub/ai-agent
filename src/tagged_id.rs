/**
 * Tagged ID encoding compatible with the API's tagged_id.py format.
 *
 * Produces IDs like "user_01PaGUP2rbg1XDh7Z9W1CEpd" from a UUID string.
 * The format is: {tag}_{version}{base58(uuid_as_128bit_int)}
 *
 * This must stay in sync with api/api/common/utils/tagged_id.py.
 */

const BASE_58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const VERSION: &str = "01";
// ceil(128 / log2(58)) = 22
const ENCODED_LENGTH: usize = 22;

/**
 * Encode a 128-bit unsigned integer as a fixed-length base58 string.
 */
fn base58_encode(n: u128) -> String {
    let base = BASE_58_CHARS.len() as u128;
    let mut result: Vec<char> = vec![BASE_58_CHARS.chars().next().unwrap(); ENCODED_LENGTH];
    let mut i = ENCODED_LENGTH as i32 - 1;
    let mut value = n;

    while value > 0 {
        let rem = (value % base) as usize;
        result[i as usize] = BASE_58_CHARS.chars().nth(rem).unwrap();
        value = value / base;
        i -= 1;
    }

    result.into_iter().collect()
}

/**
 * Parse a UUID string (with or without hyphens) into a 128-bit u128.
 */
fn uuid_to_u128(uuid: &str) -> Result<u128, String> {
    let hex = uuid.replace('-', "");
    if hex.len() != 32 {
        return Err(format!("Invalid UUID hex length: {}", hex.len()));
    }
    u128::from_str_radix(&hex, 16).map_err(|e| e.to_string())
}

/**
 * Convert an account UUID to a tagged ID in the API's format.
 *
 * @param tag - The tag prefix (e.g. "user", "org")
 * @param uuid - A UUID string (with or without hyphens)
 * @returns Tagged ID string like "user_01PaGUP2rbg1XDh7Z9W1CEpd"
 */
pub fn to_tagged_id(tag: &str, uuid: &str) -> Result<String, String> {
    let n = uuid_to_u128(uuid)?;
    Ok(format!("{}_{}{}", tag, VERSION, base58_encode(n)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base58_encode() {
        // Test basic encoding
        let encoded = base58_encode(0);
        assert_eq!(encoded.len(), ENCODED_LENGTH);
    }

    #[test]
    fn test_tagged_id() {
        // Test with a known UUID
        let result = to_tagged_id("user", "00000000-0000-0000-0000-000000000001");
        assert!(result.is_ok());
        let id = result.unwrap();
        assert!(id.starts_with("user_01"));
    }
}
