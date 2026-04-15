use serde::{Deserialize, Serialize};

static DENIALS: std::sync::RwLock<Vec<AutoModeDenial>> = std::sync::RwLock::new(Vec::new());
const MAX_DENIALS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModeDenial {
    pub tool_name: String,
    pub display: String,
    pub reason: String,
    pub timestamp: u64,
}

pub fn record_auto_mode_denial(denial: AutoModeDenial) {
    let mut denials = DENIALS.write().unwrap();
    denials.insert(0, denial);
    if denials.len() > MAX_DENIALS {
        denials.truncate(MAX_DENIALS);
    }
}

pub fn get_auto_mode_denials() -> Vec<AutoModeDenial> {
    DENIALS.read().unwrap().clone()
}

pub fn clear_auto_mode_denials() {
    DENIALS.write().unwrap().clear();
}
