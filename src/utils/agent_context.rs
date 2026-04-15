// Source: ~/claudecode/openclaudecode/src/utils/agentContext.ts
use std::collections::HashMap;

pub type AgentContext = HashMap<String, serde_json::Value>;

pub fn get_agent_context() -> AgentContext {
    AgentContext::new()
}

pub fn set_agent_context(_key: &str, _value: serde_json::Value) {}

pub fn clear_agent_context() {}
