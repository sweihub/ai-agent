use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AddressScheme {
    Uds,
    Bridge,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAddress {
    pub scheme: AddressScheme,
    pub target: String,
}

/// Parse a URI-style address into scheme + target.
pub fn parse_address(to: &str) -> ParsedAddress {
    if let Some(target) = to.strip_prefix("uds:") {
        return ParsedAddress {
            scheme: AddressScheme::Uds,
            target: target.to_string(),
        };
    }
    if let Some(target) = to.strip_prefix("bridge:") {
        return ParsedAddress {
            scheme: AddressScheme::Bridge,
            target: target.to_string(),
        };
    }
    // Legacy: old-code UDS senders emit bare socket paths in from=;
    if to.starts_with('/') {
        return ParsedAddress {
            scheme: AddressScheme::Uds,
            target: to.to_string(),
        };
    }
    ParsedAddress {
        scheme: AddressScheme::Other,
        target: to.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uds() {
        let result = parse_address("uds:/tmp/socket");
        assert_eq!(result.scheme, AddressScheme::Uds);
        assert_eq!(result.target, "/tmp/socket");
    }

    #[test]
    fn test_parse_bridge() {
        let result = parse_address("bridge:session-123");
        assert_eq!(result.scheme, AddressScheme::Bridge);
        assert_eq!(result.target, "session-123");
    }

    #[test]
    fn test_parse_other() {
        let result = parse_address("some-target");
        assert_eq!(result.scheme, AddressScheme::Other);
        assert_eq!(result.target, "some-target");
    }
}
