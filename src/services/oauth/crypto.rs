// Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/crypto.ts
//! PKCE (Proof Key for Code Exchange) helpers.
//!
//! Implements RFC 7636: code_verifier generation, SHA-256 code_challenge computation,
//! and base64url encoding with URL-safe characters.

use base64::Engine;
use rand;
use sha2::{Digest, Sha256};

/// Encodes a byte buffer to base64url encoding (RFC 4648).
/// Removes padding, replaces '+' with '-', '/' with '_'.
fn base64url_encode(buffer: &[u8]) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(buffer);
    encoded
        .replace('+', "-")
        .replace('/', "_")
        .replace('=', "")
}

/// Generates a cryptographically random code verifier.
/// The verifier is a 32-byte random string base64url-encoded (43 chars).
///
/// Returns a string of 43 to 128 printable ASCII characters.
pub fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::Rng::fill(&mut rand::thread_rng(), &mut bytes);
    base64url_encode(&bytes)
}

/// Generates a code challenge from a code verifier using SHA-256.
/// This is used in the authorization request as `code_challenge`.
pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    base64url_encode(&digest)
}

/// Generates a cryptographically random state parameter for CSRF protection.
/// Returns a 32-byte random string base64url-encoded (43 chars).
pub fn generate_state() -> String {
    let mut bytes = [0u8; 32];
    rand::Rng::fill(&mut rand::thread_rng(), &mut bytes);
    base64url_encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_verifier_length() {
        let verifier = generate_code_verifier();
        // 32 bytes base64 encoded = 44 chars, minus padding = 43 chars
        assert!(verifier.len() >= 43);
        assert!(verifier.len() <= 128);
        // Should only contain URL-safe characters
        assert!(verifier.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_code_challenge_from_verifier() {
        let verifier = generate_code_verifier();
        let challenge = generate_code_challenge(&verifier);
        // SHA-256 digest is 32 bytes, base64url encoded = 44 chars, minus padding = 43 chars
        assert!(challenge.len() >= 43);
        assert!(challenge.len() <= 128);
        assert!(challenge.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_code_challenge_deterministic() {
        let verifier = "test-verifier-12345";
        let challenge1 = generate_code_challenge(verifier);
        let challenge2 = generate_code_challenge(verifier);
        assert_eq!(challenge1, challenge2);
    }

    #[test]
    fn test_state_length() {
        let state = generate_state();
        assert_eq!(state.len(), 43);
    }

    #[test]
    fn test_state_unique() {
        let state1 = generate_state();
        let state2 = generate_state();
        assert_ne!(state1, state2);
    }

    #[test]
    fn test_base64url_no_padding() {
        // Verify that padding characters are stripped
        let verifier = generate_code_verifier();
        assert!(!verifier.contains('='));
        assert!(!verifier.contains('+'));
        assert!(!verifier.contains('/'));
    }
}
