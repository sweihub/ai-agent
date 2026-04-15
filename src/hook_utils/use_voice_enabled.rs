use std::sync::atomic::{AtomicBool, Ordering};

pub struct VoiceEnabledState {
    enabled: AtomicBool,
    audio_input_available: AtomicBool,
    audio_output_available: AtomicBool,
}

impl VoiceEnabledState {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            audio_input_available: AtomicBool::new(false),
            audio_output_available: AtomicBool::new(false),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn set_audio_input_available(&self, available: bool) {
        self.audio_input_available
            .store(available, Ordering::SeqCst);
    }

    pub fn is_audio_input_available(&self) -> bool {
        self.audio_input_available.load(Ordering::SeqCst)
    }

    pub fn set_audio_output_available(&self, available: bool) {
        self.audio_output_available
            .store(available, Ordering::SeqCst);
    }

    pub fn is_audio_output_available(&self) -> bool {
        self.audio_output_available.load(Ordering::SeqCst)
    }

    pub fn can_use_voice(&self) -> bool {
        self.is_enabled() && self.is_audio_input_available()
    }
}

impl Default for VoiceEnabledState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_enabled_state() {
        let state = VoiceEnabledState::new();

        assert!(!state.is_enabled());

        state.set_enabled(true);
        assert!(state.is_enabled());
    }

    #[test]
    fn test_audio_availability() {
        let state = VoiceEnabledState::new();

        state.set_audio_input_available(true);
        state.set_audio_output_available(true);

        assert!(state.is_audio_input_available());
        assert!(state.is_audio_output_available());
    }

    #[test]
    fn test_can_use_voice() {
        let state = VoiceEnabledState::new();

        state.set_enabled(true);
        state.set_audio_input_available(true);

        assert!(state.can_use_voice());

        state.set_enabled(false);
        assert!(!state.can_use_voice());
    }
}
