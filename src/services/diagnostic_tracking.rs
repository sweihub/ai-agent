//! Diagnostic tracking service - tracks IDE diagnostics before/after edits.
//!
//! Translates diagnosticTracking.ts from claude code.

use std::collections::HashMap;

pub const MAX_DIAGNOSTICS_SUMMARY_CHARS: usize = 4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct DiagnosticRange {
    pub start: DiagnosticPosition,
    pub end: DiagnosticPosition,
}

#[derive(Debug, Clone)]
pub struct DiagnosticPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub range: DiagnosticRange,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticFile {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct DiagnosticTrackingService {
    initialized: bool,
    baseline: HashMap<String, Vec<Diagnostic>>,
    right_file_diagnostics_state: HashMap<String, Vec<Diagnostic>>,
    last_processed_timestamps: HashMap<String, u64>,
}

impl DiagnosticTrackingService {
    pub fn new() -> Self {
        Self {
            initialized: false,
            baseline: HashMap::new(),
            right_file_diagnostics_state: HashMap::new(),
            last_processed_timestamps: HashMap::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    pub fn shutdown(&mut self) {
        self.initialized = false;
        self.baseline.clear();
        self.right_file_diagnostics_state.clear();
        self.last_processed_timestamps.clear();
    }

    pub fn reset(&mut self) {
        self.baseline.clear();
        self.right_file_diagnostics_state.clear();
        self.last_processed_timestamps.clear();
    }

    fn normalize_file_uri(&self, file_uri: &str) -> String {
        let prefixes = ["file://", "_claude_fs_right:", "_claude_fs_left:"];

        let mut normalized = file_uri.to_string();
        for prefix in &prefixes {
            if file_uri.starts_with(prefix) {
                normalized = file_uri
                    .strip_prefix(prefix)
                    .unwrap_or(file_uri)
                    .to_string();
                break;
            }
        }

        normalized
    }

    pub fn before_file_edited(&mut self, file_path: &str, timestamp: u64) {
        if !self.initialized {
            return;
        }

        let normalized_path = self.normalize_file_uri(file_path);
        self.baseline.insert(normalized_path.clone(), Vec::new());
        self.last_processed_timestamps
            .insert(normalized_path, timestamp);
    }

    pub fn set_baseline_diagnostics(&mut self, file_path: &str, diagnostics: Vec<Diagnostic>) {
        let normalized_path = self.normalize_file_uri(file_path);
        self.baseline.insert(normalized_path, diagnostics);
    }

    pub fn get_baseline(&self, file_path: &str) -> Option<&Vec<Diagnostic>> {
        let normalized_path = self.normalize_file_uri(file_path);
        self.baseline.get(&normalized_path)
    }

    fn are_diagnostics_equal(&self, a: &Diagnostic, b: &Diagnostic) -> bool {
        a.message == b.message
            && a.severity == b.severity
            && a.source == b.source
            && a.code == b.code
            && a.range.start.line == b.range.start.line
            && a.range.start.character == b.range.start.character
            && a.range.end.line == b.range.end.line
            && a.range.end.character == b.range.end.character
    }

    fn are_diagnostic_arrays_equal(&self, a: &[Diagnostic], b: &[Diagnostic]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a.iter().all(|diag_a| {
            b.iter()
                .any(|diag_b| self.are_diagnostics_equal(diag_a, diag_b))
        }) && b.iter().all(|diag_b| {
            a.iter()
                .any(|diag_a| self.are_diagnostics_equal(diag_a, diag_b))
        })
    }

    pub fn format_diagnostics_summary(files: &[DiagnosticFile]) -> String {
        let truncation_marker = "…[truncated]";

        let result: String = files
            .iter()
            .map(|file| {
                let filename = file.uri.split('/').last().unwrap_or(&file.uri);
                let diagnostics: String = file
                    .diagnostics
                    .iter()
                    .map(|d| {
                        let severity_symbol = Self::get_severity_symbol(&d.severity);
                        let code_str = d
                            .code
                            .as_ref()
                            .map(|c| format!(" [{}]", c))
                            .unwrap_or_default();
                        let source_str = d
                            .source
                            .as_ref()
                            .map(|s| format!(" ({})", s))
                            .unwrap_or_default();
                        format!(
                            "  {} [Line {}:{}] {}{}{}",
                            severity_symbol,
                            d.range.start.line + 1,
                            d.range.start.character + 1,
                            d.message,
                            code_str,
                            source_str
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                format!("{}:\n{}", filename, diagnostics)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        if result.len() > MAX_DIAGNOSTICS_SUMMARY_CHARS {
            return result[..MAX_DIAGNOSTICS_SUMMARY_CHARS - truncation_marker.len()].to_string()
                + truncation_marker;
        }

        result
    }

    pub fn get_severity_symbol(severity: &DiagnosticSeverity) -> &'static str {
        match severity {
            DiagnosticSeverity::Error => "✗",
            DiagnosticSeverity::Warning => "⚠",
            DiagnosticSeverity::Info => "ℹ",
            DiagnosticSeverity::Hint => "★",
        }
    }
}

impl Default for DiagnosticTrackingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_file_uri() {
        let service = DiagnosticTrackingService::new();

        assert_eq!(
            service.normalize_file_uri("file:///path/to/file.ts"),
            "/path/to/file.ts"
        );
        assert_eq!(
            service.normalize_file_uri("_claude_fs_right:/path/to/file.ts"),
            "/path/to/file.ts"
        );
    }

    #[test]
    fn test_severity_symbols() {
        assert_eq!(
            DiagnosticTrackingService::get_severity_symbol(&DiagnosticSeverity::Error),
            "✗"
        );
        assert_eq!(
            DiagnosticTrackingService::get_severity_symbol(&DiagnosticSeverity::Warning),
            "⚠"
        );
        assert_eq!(
            DiagnosticTrackingService::get_severity_symbol(&DiagnosticSeverity::Info),
            "ℹ"
        );
        assert_eq!(
            DiagnosticTrackingService::get_severity_symbol(&DiagnosticSeverity::Hint),
            "★"
        );
    }

    #[test]
    fn test_diagnostic_tracking_service() {
        let mut service = DiagnosticTrackingService::new();

        assert!(!service.initialized);

        service.initialize();
        assert!(service.initialized);

        service.reset();
        assert!(service.baseline.is_empty());

        service.shutdown();
        assert!(!service.initialized);
    }
}
