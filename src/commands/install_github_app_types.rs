use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow(pub serde_json::Value);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning(pub serde_json::Value);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State(pub serde_json::Value);
