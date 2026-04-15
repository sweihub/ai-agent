#![allow(dead_code)]

pub fn get_agent_definitions() -> Vec<AgentDefinition> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub model: Option<String>,
}
