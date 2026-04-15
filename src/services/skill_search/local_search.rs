use std::collections::HashMap;

pub fn search_local_skills(_query: &str) -> Vec<SkillMatch> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct SkillMatch {
    pub name: String,
    pub score: f32,
}
