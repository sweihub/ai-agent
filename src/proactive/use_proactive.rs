pub fn use_proactive() -> ProactiveState {
    ProactiveState {
        active: false,
        paused: false,
    }
}

#[derive(Debug, Clone)]
pub struct ProactiveState {
    pub active: bool,
    pub paused: bool,
}
