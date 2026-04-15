// Source: ~/claudecode/openclaudecode/src/utils/agentId.ts
//! Deterministic Agent ID System
//!
//! This module provides helper functions for formatting and parsing deterministic
//! agent IDs used in the swarm/teammate system.
//!
//! ## ID Formats
//!
//! **Agent IDs**: `agentName@teamName`
//! - Example: `team-lead@my-project`, `researcher@my-project`
//! - The @ symbol acts as a separator between agent name and team name
//!
//! **Request IDs**: `{requestType}-{timestamp}@{agentId}`
//! - Example: `shutdown-1702500000000@researcher@my-project`
//! - Used for shutdown requests, plan approvals, etc.
//!
//! ## Why Deterministic IDs?
//!
//! Deterministic IDs provide several benefits:
//!
//! 1. **Reproducibility**: The same agent spawned with the same name in the same team
//!    always gets the same ID, enabling reconnection after crashes/restarts.
//!
//! 2. **Human-readable**: IDs are meaningful and debuggable (e.g., `tester@my-project`).
//!
//! 3. **Predictable**: Team leads can compute a teammate's ID without looking it up,
//!    simplifying message routing and task assignment.
//!
//! ## Constraints
//!
//! - Agent names must NOT contain `@` (it's used as the separator)
//! - Use `sanitize_agent_name()` from TeammateTool to strip @ from names

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAgentId {
    pub agent_name: String,
    pub team_name: String,
}

/// Formats an agent ID in the format `agentName@teamName`.
pub fn format_agent_id(agent_name: &str, team_name: &str) -> String {
    format!("{agent_name}@{team_name}")
}

/// Parses an agent ID into its components.
/// Returns None if the ID doesn't contain the @ separator.
pub fn parse_agent_id(agent_id: &str) -> Option<ParsedAgentId> {
    let at_index = agent_id.find('@')?;
    Some(ParsedAgentId {
        agent_name: agent_id[..at_index].to_string(),
        team_name: agent_id[at_index + 1..].to_string(),
    })
}

/// Formats a request ID in the format `{requestType}-{timestamp}@{agentId}`.
pub fn generate_request_id(request_type: &str, agent_id: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{request_type}-{timestamp}@{agent_id}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequestId {
    pub request_type: String,
    pub timestamp: u128,
    pub agent_id: String,
}

/// Parses a request ID into its components.
/// Returns None if the request ID doesn't match the expected format.
pub fn parse_request_id(request_id: &str) -> Option<ParsedRequestId> {
    let at_index = request_id.find('@')?;

    let prefix = &request_id[..at_index];
    let agent_id = &request_id[at_index + 1..];

    let last_dash_index = prefix.rfind('-')?;

    let request_type = prefix[..last_dash_index].to_string();
    let timestamp_str = &prefix[last_dash_index + 1..];
    let timestamp: u128 = timestamp_str.parse().ok()?;

    Some(ParsedRequestId {
        request_type,
        timestamp,
        agent_id: agent_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_agent_id() {
        let id = format_agent_id("team-lead", "my-project");
        assert_eq!(id, "team-lead@my-project");
    }

    #[test]
    fn test_parse_agent_id() {
        let parsed = parse_agent_id("researcher@my-project").unwrap();
        assert_eq!(parsed.agent_name, "researcher");
        assert_eq!(parsed.team_name, "my-project");

        assert!(parse_agent_id("no-at-sign").is_none());
    }

    #[test]
    fn test_generate_and_parse_request_id() {
        let agent_id = "test@project";
        let request_id = generate_request_id("shutdown", agent_id);

        let parsed = parse_request_id(&request_id).unwrap();
        assert_eq!(parsed.request_type, "shutdown");
        assert_eq!(parsed.agent_id, agent_id);
        assert!(parsed.timestamp > 0);
    }

    #[test]
    fn test_parse_request_id_invalid() {
        assert!(parse_request_id("no-at-sign").is_none());
        assert!(parse_request_id("type-nodash@agent").is_none());
        assert!(parse_request_id("type-notanumber@agent").is_none());
    }
}
