pub fn is_claude_desktop_enabled() -> bool {
    false
}

pub fn get_claude_desktop_config() -> Option<ClaudeDesktopConfig> {
    None
}

#[derive(Clone, Debug)]
pub struct ClaudeDesktopConfig {
    pub config_path: String,
}
