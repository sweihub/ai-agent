use std::collections::HashMap;

pub fn get_remote_skill_state() -> HashMap<String, SkillState> {
    HashMap::new()
}

#[derive(Debug, Clone)]
pub struct SkillState {
    pub loaded: bool,
}
