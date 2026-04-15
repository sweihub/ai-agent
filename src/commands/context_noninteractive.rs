use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    pub categories: Vec<CategoryData>,
    pub total_tokens: u32,
    pub raw_max_tokens: u32,
    pub percentage: f32,
    pub model: String,
    pub memory_files: Vec<MemoryFile>,
    pub mcp_tools: Vec<McpTool>,
    pub agents: Vec<AgentData>,
    pub skills: SkillsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryData {
    pub name: String,
    pub tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFile {
    #[serde(rename = "type")]
    pub file_type: String,
    pub path: String,
    pub tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub server_name: String,
    pub tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentData {
    pub agent_type: String,
    pub source: String,
    pub tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsData {
    pub tokens: u32,
    pub skill_frontmatter: Vec<SkillFrontmatter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub source: String,
    pub tokens: u32,
}

pub async fn collect_context_data() -> ContextData {
    ContextData {
        categories: vec![],
        total_tokens: 0,
        raw_max_tokens: 0,
        percentage: 0.0,
        model: String::new(),
        memory_files: vec![],
        mcp_tools: vec![],
        agents: vec![],
        skills: SkillsData {
            tokens: 0,
            skill_frontmatter: vec![],
        },
    }
}
