//! Session URL utilities.

use url::Url;

/// Build a session URL from components
pub fn build_session_url(base: &str, session_id: &str) -> Result<Url, url::ParseError> {
    let mut url = Url::parse(base)?;
    url.path_segments_mut()
        .map_err(|_| url::ParseError::InvalidIpv6Address)?
        .push("session");
    url.path_segments_mut()
        .map_err(|_| url::ParseError::InvalidIpv6Address)?
        .push(session_id);
    Ok(url)
}

/// Extract session ID from URL
pub fn extract_session_id(url: &str) -> Option<String> {
    let url = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return None,
    };

    let segments: Vec<&str> = url.path_segments()?.collect();

    // Look for session ID in path
    if segments.len() >= 2 && segments[segments.len() - 2] == "session" {
        return Some(segments[segments.len() - 1].to_string());
    }

    // Look for session ID as query parameter
    url.query_pairs()
        .find(|(k, _)| k == "session")
        .map(|(_, v)| v.to_string())
}

/// Check if a URL is a valid session URL
pub fn is_valid_session_url(url: &str) -> bool {
    extract_session_id(url).is_some()
}
