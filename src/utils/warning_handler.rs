use std::collections::HashMap;

pub struct WarningHandler {
    warnings: Vec<Warning>,
}

#[derive(Debug, Clone)]
pub struct Warning {
    pub message: String,
    pub code: Option<String>,
    pub count: usize,
}

impl WarningHandler {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, message: String, code: Option<String>) {
        if let Some(existing) = self.warnings.iter_mut().find(|w| w.message == message) {
            existing.count += 1;
        } else {
            self.warnings.push(Warning {
                message,
                code,
                count: 1,
            });
        }
    }

    pub fn get_warnings(&self) -> &[Warning] {
        &self.warnings
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn clear(&mut self) {
        self.warnings.clear();
    }

    pub fn format_warnings(&self) -> String {
        self.warnings
            .iter()
            .map(|w| {
                if w.count > 1 {
                    format!("{} ({} times)", w.message, w.count)
                } else {
                    w.message.clone()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for WarningHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warning_handler() {
        let mut handler = WarningHandler::new();
        handler.add_warning("Test warning".to_string(), Some("W001".to_string()));
        handler.add_warning("Test warning".to_string(), Some("W001".to_string()));

        assert_eq!(handler.get_warnings().len(), 1);
        assert_eq!(handler.get_warnings()[0].count, 2);
    }
}
