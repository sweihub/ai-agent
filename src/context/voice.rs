// Source: /data/home/swei/claudecode/openclaudecode/src/commands/voice/voice.ts
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceContext {
    pub is_enabled: bool,
    pub is_listening: bool,
    pub transcription: Option<String>,
    pub confidence: Option<f32>,
}

impl VoiceContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
        self.is_listening = false;
    }

    pub fn start_listening(&mut self) {
        if self.is_enabled {
            self.is_listening = true;
        }
    }

    pub fn stop_listening(&mut self) {
        self.is_listening = false;
    }

    pub fn set_transcription(&mut self, text: String, confidence: f32) {
        self.transcription = Some(text);
        self.confidence = Some(confidence);
    }

    pub fn clear_transcription(&mut self) {
        self.transcription = None;
        self.confidence = None;
    }
}
