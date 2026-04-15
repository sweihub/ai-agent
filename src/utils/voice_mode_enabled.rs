#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceModeState {
    Idle,
    Recording,
    Processing,
}

pub fn is_voice_enabled() -> bool {
    false
}

pub fn set_voice_state(_state: VoiceModeState) {}

pub fn get_audio_levels() -> Vec<f32> {
    vec![]
}
