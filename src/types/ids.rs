// Source: ~/claudecode/openclaudecode/src/types/ids.ts

//! Branded types for session and agent IDs.
//! These prevent accidentally mixing up session IDs and agent IDs at compile time.

use serde::{Deserialize, Serialize};

/// A session ID uniquely identifies a Claude Code session.
/// Returned by getSessionId().
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

/// An agent ID uniquely identifies a subagent within a session.
/// Returned by createAgentId().
/// When present, indicates the context is a subagent (not the main session).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

/// Cast a raw string to SessionId.
/// Use sparingly - prefer getSessionId() when possible.
pub fn as_session_id(id: &str) -> SessionId {
    SessionId(id.to_string())
}

/// Cast a raw string to AgentId.
/// Use sparingly - prefer createAgentId() when possible.
pub fn as_agent_id(id: &str) -> AgentId {
    AgentId(id.to_string())
}

const AGENT_ID_PATTERN: &str = r"^a(?:.+-)?[0-9a-f]{16}$";

/// Validate and brand a string as AgentId.
/// Matches the format produced by createAgentId(): `a` + optional `<label>-` + 16 hex chars.
/// Returns None if the string doesn't match (e.g. teammate names, team-addressing).
pub fn to_agent_id(s: &str) -> Option<AgentId> {
    let re = regex::Regex::new(AGENT_ID_PATTERN).ok()?;
    if re.is_match(s) {
        Some(AgentId(s.to_string()))
    } else {
        None
    }
}

impl SessionId {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AgentId {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        as_session_id(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        as_agent_id(s)
    }
}
