//! Tool schema types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: InputSchema,
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone)]
pub struct InputSchema {
    pub type_: String,
    pub properties: HashMap<String, Property>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub type_: String,
    pub description: Option<String>,
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum CacheControl {
    Ephemeral,
}
