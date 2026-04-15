#[derive(Debug, Clone, PartialEq)]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub severity: FindingSeverity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub rule: Option<String>,
}

impl Finding {
    pub fn info(message: &str) -> Self {
        Self {
            severity: FindingSeverity::Info,
            message: message.to_string(),
            file: None,
            line: None,
            rule: None,
        }
    }

    pub fn warning(message: &str) -> Self {
        Self {
            severity: FindingSeverity::Warning,
            message: message.to_string(),
            file: None,
            line: None,
            rule: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            severity: FindingSeverity::Error,
            message: message.to_string(),
            file: None,
            line: None,
            rule: None,
        }
    }

    pub fn with_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());
        self
    }

    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_rule(mut self, rule: &str) -> Self {
        self.rule = Some(rule.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub findings: Vec<Finding>,
    pub summary: String,
    pub score: Option<u32>,
}

impl ReviewResult {
    pub fn new(findings: Vec<Finding>, summary: &str) -> Self {
        let score = Self::calculate_score(&findings);
        Self {
            findings,
            summary: summary.to_string(),
            score: Some(score),
        }
    }

    fn calculate_score(findings: &[Finding]) -> u32 {
        let mut score = 100u32;
        for finding in findings {
            match finding.severity {
                FindingSeverity::Info => score -= 1,
                FindingSeverity::Warning => score -= 5,
                FindingSeverity::Error => score -= 10,
                FindingSeverity::Critical => score -= 25,
            }
        }
        score.saturating_sub(0)
    }

    pub fn error_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| {
                matches!(
                    f.severity,
                    FindingSeverity::Error | FindingSeverity::Critical
                )
            })
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| matches!(f.severity, FindingSeverity::Warning))
            .count()
    }
}

pub fn review_code(code: &str, language: &str) -> ReviewResult {
    let mut findings = Vec::new();

    match language {
        "rust" => {
            if code.contains("unsafe") {
                findings.push(
                    Finding::warning("Use of unsafe code detected").with_rule("security/unsafe"),
                );
            }
            if code.contains("expect(") || code.contains("unwrap(") {
                findings.push(
                    Finding::warning("Potential panic with expect/unwrap")
                        .with_rule("style/expect"),
                );
            }
        }
        "typescript" | "javascript" => {
            if code.contains("eval(") {
                findings
                    .push(Finding::error("Use of eval() is dangerous").with_rule("security/eval"));
            }
            if code.contains("console.log") && code.contains("TODO") {
                findings.push(Finding::info("Debug logging left in code").with_rule("style/debug"));
            }
        }
        _ => {}
    }

    if code.len() > 10000 {
        findings.push(
            Finding::warning("File is very large, consider splitting")
                .with_rule("maintainability size"),
        );
    }

    let summary = format!(
        "Found {} issues: {} errors, {} warnings",
        findings.len(),
        findings
            .iter()
            .filter(|f| matches!(
                f.severity,
                FindingSeverity::Error | FindingSeverity::Critical
            ))
            .count(),
        findings
            .iter()
            .filter(|f| matches!(f.severity, FindingSeverity::Warning))
            .count()
    );

    ReviewResult::new(findings, &summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_creation() {
        let finding = Finding::error("Test error")
            .with_file("src/main.rs")
            .with_line(42)
            .with_rule("test/rule");

        assert_eq!(finding.severity, FindingSeverity::Error);
        assert_eq!(finding.file, Some("src/main.rs".to_string()));
        assert_eq!(finding.line, Some(42));
    }

    #[test]
    fn test_review_result_score() {
        let findings = vec![
            Finding::error("Error 1"),
            Finding::warning("Warning 1"),
            Finding::info("Info 1"),
        ];
        let result = ReviewResult::new(findings, "Test review");
        assert_eq!(result.score, Some(84));
    }
}
