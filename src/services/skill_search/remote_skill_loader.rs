pub fn load_remote_skill(_skill_id: &str) -> Option<RemoteSkill> {
    None
}

#[derive(Debug, Clone)]
pub struct RemoteSkill {
    pub id: String,
    pub name: String,
}
