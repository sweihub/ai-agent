use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceState {
    pub enabled: bool,
    pub mode: VoiceMode,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VoiceMode {
    PushToTalk,
    VoiceActivity,
    Continuous,
}

impl Default for VoiceState {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: VoiceMode::PushToTalk,
            input_device: None,
            output_device: None,
        }
    }
}

impl VoiceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn set_mode(&mut self, mode: VoiceMode) {
        self.mode = mode;
    }

    pub fn set_input_device(&mut self, device: String) {
        self.input_device = Some(device);
    }

    pub fn set_output_device(&mut self, device: String) {
        self.output_device = Some(device);
    }
}
