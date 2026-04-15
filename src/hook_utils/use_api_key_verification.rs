#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    Loading,
    Valid,
    Invalid,
    Missing,
    Error,
}

pub struct ApiKeyVerificationState {
    pub status: VerificationStatus,
    pub error: Option<String>,
}

impl ApiKeyVerificationState {
    pub fn new() -> Self {
        Self {
            status: VerificationStatus::Loading,
            error: None,
        }
    }

    pub fn set_valid(&mut self) {
        self.status = VerificationStatus::Valid;
        self.error = None;
    }

    pub fn set_invalid(&mut self) {
        self.status = VerificationStatus::Invalid;
    }

    pub fn set_missing(&mut self) {
        self.status = VerificationStatus::Missing;
    }

    pub fn set_error(&mut self, error: String) {
        self.status = VerificationStatus::Error;
        self.error = Some(error);
    }

    pub fn is_valid(&self) -> bool {
        self.status == VerificationStatus::Valid
    }
}

impl Default for ApiKeyVerificationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_verification_state() {
        let mut state = ApiKeyVerificationState::new();
        assert_eq!(state.status, VerificationStatus::Loading);

        state.set_valid();
        assert!(state.is_valid());
    }
}
