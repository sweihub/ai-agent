// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/oauthPort.ts
//! OAuth redirect port helpers

use std::env;

// Windows dynamic port range 49152-65535 is reserved (RFC 6335)
// Windows client uses 39152-49151 per Microsoft docs
const REDIRECT_PORT_FALLBACK: u16 = 3118;

fn redirect_port_range() -> (u16, u16) {
    if cfg!(target_os = "windows") {
        (39152, 49151)
    } else {
        (49152, 65535)
    }
}

/// Builds a redirect URI on localhost with the given port and a fixed `/callback` path.
/// RFC 8252 Section 7.3 (OAuth for Native Apps): loopback redirect URIs match any
/// port as long as the path matches.
pub fn build_redirect_uri(port: Option<u16>) -> String {
    let port = port.unwrap_or(REDIRECT_PORT_FALLBACK);
    format!("http://localhost:{}/callback", port)
}

fn get_mcp_oauth_callback_port() -> Option<u16> {
    env::var("MCP_OAUTH_CALLBACK_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .filter(|&p| p > 0)
}

/// Finds an available port in the specified range for OAuth redirect
/// Uses random selection for better security
pub fn find_available_port() -> Option<u16> {
    // First, try the configured port if specified
    if let Some(port) = get_mcp_oauth_callback_port() {
        return Some(port);
    }

    let (min, max) = redirect_port_range();
    let range = max - min + 1;
    let max_attempts = 100.min(range as usize);

    for _ in 0..max_attempts {
        let port = min + (rand_u16() % range);

        if is_port_available(port) {
            return Some(port);
        }
    }

    // If random selection failed, try the fallback port
    if is_port_available(REDIRECT_PORT_FALLBACK) {
        return Some(REDIRECT_PORT_FALLBACK);
    }

    None
}

/// Simple random u16 using system time
fn rand_u16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 65536) as u16
}

/// Check if a port is available
fn is_port_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_redirect_uri() {
        let uri = build_redirect_uri(Some(8080));
        assert_eq!(uri, "http://localhost:8080/callback");
    }

    #[test]
    fn test_find_available_port() {
        let port = find_available_port();
        assert!(port.is_some());
    }
}
