// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/oauthPort.ts
//! OAuth redirect port management.
//!
//! RFC 8252 Section 7.3 (OAuth for Native Apps): loopback redirect URIs
//! match any port as long as the path matches.
//!
//! Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/oauthPort.ts

use rand;
use std::net::ToSocketAddrs;
use tokio::net::TcpListener;

/// Fallback port for OAuth redirect if no port is available.
const REDIRECT_PORT_FALLBACK: u16 = 3118;

/// Build a redirect URI on localhost with the given port and a fixed /callback path.
pub fn build_redirect_uri(port: u16) -> String {
    format!("http://localhost:{port}/callback")
}

/// Get the configured MCP OAuth callback port from environment.
fn get_mcp_oauth_callback_port() -> Option<u16> {
    std::env::var("AI_CODE_MCP_OAUTH_CALLBACK_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|p| *p > 0)
}

/// Finds an available port in the ephemeral range for OAuth redirect.
/// Uses random selection for better security.
pub async fn find_available_port() -> anyhow::Result<u16> {
    // First, try the configured port if specified
    if let Some(configured) = get_mcp_oauth_callback_port() {
        return Ok(configured);
    }

    // Ephemeral port range (IANA recommended): 49152-65535
    const MIN_PORT: u16 = 49152;
    const MAX_PORT: u16 = 65535;
    let range = (MAX_PORT - MIN_PORT + 1) as u32;
    let max_attempts = std::cmp::min(range, 100); // Don't try forever

    let mut rng = rand::thread_rng();

    for _ in 0..max_attempts {
        let port = MIN_PORT + (rand::random::<u32>() % range) as u16;

        if is_port_available(port).await {
            return Ok(port);
        }
    }

    // If random selection failed, try the fallback port
    if is_port_available(REDIRECT_PORT_FALLBACK).await {
        return Ok(REDIRECT_PORT_FALLBACK);
    }

    Err(anyhow::anyhow!("No available ports for OAuth redirect"))
}

async fn is_port_available(port: u16) -> bool {
    let addr = format!("127.0.0.1:{port}");
    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            // Port is available, close immediately
            drop(listener);
            true
        }
        Err(_) => false,
    }
}
