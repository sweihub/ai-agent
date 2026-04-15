//! Validation error types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: Option<String>,
}

impl ValidationError {
    pub fn new(field: String, message: String) -> Self {
        Self {
            field,
            message,
            code: None,
        }
    }

    pub fn with_code(field: String, message: String, code: String) -> Self {
        Self {
            field,
            message,
            code: Some(code),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}
