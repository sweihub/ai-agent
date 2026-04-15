// Source: /data/home/swei/claudecode/openclaudecode/src/commands/doctor/doctor.tsx
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorScreen {
    pub diagnostics: Vec<DiagnosticResult>,
    pub is_loading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub name: String,
    pub status: DiagnosticStatus,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

impl DoctorScreen {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            is_loading: false,
        }
    }

    pub fn add_diagnostic(&mut self, result: DiagnosticResult) {
        self.diagnostics.push(result);
    }

    pub fn loading(&mut self) {
        self.is_loading = true;
    }

    pub fn finished(&mut self) {
        self.is_loading = false;
    }

    pub fn pass_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.status == DiagnosticStatus::Pass)
            .count()
    }

    pub fn fail_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.status == DiagnosticStatus::Fail)
            .count()
    }
}

impl Default for DoctorScreen {
    fn default() -> Self {
        Self::new()
    }
}
