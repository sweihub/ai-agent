// Source: /data/home/swei/claudecode/openclaudecode/src/commands/voice/voice.ts
//! Voice service - audio recording for push-to-talk voice input.
//!
////! Translates voice.ts from claude code.

pub const RECORDING_SAMPLE_RATE: u32 = 16000;
pub const RECORDING_CHANNELS: u32 = 1;

pub const SILENCE_DURATION_SECS: &str = "2.0";
pub const SILENCE_THRESHOLD: &str = "3%";

#[derive(Debug, Clone)]
pub struct RecordingAvailability {
    pub available: bool,
    pub reason: Option<String>,
}

impl RecordingAvailability {
    pub fn available() -> Self {
        Self {
            available: true,
            reason: None,
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            available: false,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoiceDependencies {
    pub available: bool,
    pub missing: Vec<String>,
    pub install_command: Option<String>,
}

impl VoiceDependencies {
    pub fn available() -> Self {
        Self {
            available: true,
            missing: Vec::new(),
            install_command: None,
        }
    }

    pub fn missing_deps(missing: Vec<String>, install_command: Option<String>) -> Self {
        Self {
            available: missing.is_empty(),
            missing,
            install_command,
        }
    }
}

pub fn has_command(_cmd: &str) -> bool {
    false
}

pub fn _reset_arecord_probe_for_testing() {}

pub fn _reset_alsa_cards_for_testing() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_availability_available() {
        let avail = RecordingAvailability::available();
        assert!(avail.available);
        assert!(avail.reason.is_none());
    }

    #[test]
    fn test_recording_availability_unavailable() {
        let avail = RecordingAvailability::unavailable("No microphone");
        assert!(!avail.available);
        assert_eq!(avail.reason, Some("No microphone".to_string()));
    }

    #[test]
    fn test_voice_dependencies_available() {
        let deps = VoiceDependencies::available();
        assert!(deps.available);
        assert!(deps.missing.is_empty());
    }

    #[test]
    fn test_voice_dependencies_missing() {
        let deps = VoiceDependencies::missing_deps(
            vec!["sox".to_string()],
            Some("brew install sox".to_string()),
        );
        assert!(!deps.available);
        assert_eq!(deps.missing.len(), 1);
        assert!(deps.install_command.is_some());
    }
}
