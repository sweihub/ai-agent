// Source: ~/claudecode/openclaudecode/src/utils/plugins/validatePlugin.ts
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use super::schemas::{PluginManifest, PluginMarketplaceEntry};
use super::types::PluginMarketplace;

pub struct ValidationResult {
    pub success: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub file_path: String,
    pub file_type: String,
}

#[derive(Debug)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub path: String,
    pub message: String,
}

/// Validate a plugin manifest file (plugin.json).
pub async fn validate_plugin_manifest(file_path: &str) -> ValidationResult {
    let absolute_path = match std::fs::canonicalize(file_path) {
        Ok(p) => p,
        Err(e) => {
            return ValidationResult {
                success: false,
                errors: vec![ValidationError {
                    path: "file".to_string(),
                    message: format!("File not found: {}", e),
                }],
                warnings: Vec::new(),
                file_path: file_path.to_string(),
                file_type: "plugin".to_string(),
            };
        }
    };

    let content = match tokio::fs::read_to_string(&absolute_path).await {
        Ok(c) => c,
        Err(e) => {
            return ValidationResult {
                success: false,
                errors: vec![ValidationError {
                    path: "file".to_string(),
                    message: format!("Failed to read file: {}", e),
                }],
                warnings: Vec::new(),
                file_path: absolute_path.to_string_lossy().to_string(),
                file_type: "plugin".to_string(),
            };
        }
    };

    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return ValidationResult {
                success: false,
                errors: vec![ValidationError {
                    path: "json".to_string(),
                    message: format!("Invalid JSON syntax: {}", e),
                }],
                warnings: Vec::new(),
                file_path: absolute_path.to_string_lossy().to_string(),
                file_type: "plugin".to_string(),
            };
        }
    };

    let manifest: PluginManifest = match serde_json::from_value(parsed) {
        Ok(m) => m,
        Err(e) => {
            return ValidationResult {
                success: false,
                errors: e
                    .to_string()
                    .lines()
                    .map(|line| ValidationError {
                        path: "manifest".to_string(),
                        message: line.to_string(),
                    })
                    .collect(),
                warnings: Vec::new(),
                file_path: absolute_path.to_string_lossy().to_string(),
                file_type: "plugin".to_string(),
            };
        }
    };

    let mut warnings = Vec::new();

    // Warn if name isn't kebab-case
    if !regex::Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$")
        .unwrap()
        .is_match(&manifest.name)
    {
        warnings.push(ValidationWarning {
            path: "name".to_string(),
            message: format!("Plugin name \"{}\" is not kebab-case.", manifest.name),
        });
    }

    // Warn if no version specified
    if manifest.version.is_none() {
        warnings.push(ValidationWarning {
            path: "version".to_string(),
            message: "No version specified.".to_string(),
        });
    }

    // Warn if no description
    if manifest.description.is_none() {
        warnings.push(ValidationWarning {
            path: "description".to_string(),
            message: "No description provided.".to_string(),
        });
    }

    ValidationResult {
        success: true,
        errors: Vec::new(),
        warnings,
        file_path: absolute_path.to_string_lossy().to_string(),
        file_type: "plugin".to_string(),
    }
}

/// Validate a marketplace manifest file (marketplace.json).
pub async fn validate_marketplace_manifest(file_path: &str) -> ValidationResult {
    let absolute_path = match std::fs::canonicalize(file_path) {
        Ok(p) => p,
        Err(e) => {
            return ValidationResult {
                success: false,
                errors: vec![ValidationError {
                    path: "file".to_string(),
                    message: format!("File not found: {}", e),
                }],
                warnings: Vec::new(),
                file_path: file_path.to_string(),
                file_type: "marketplace".to_string(),
            };
        }
    };

    let content = match tokio::fs::read_to_string(&absolute_path).await {
        Ok(c) => c,
        Err(e) => {
            return ValidationResult {
                success: false,
                errors: vec![ValidationError {
                    path: "file".to_string(),
                    message: format!("Failed to read file: {}", e),
                }],
                warnings: Vec::new(),
                file_path: absolute_path.to_string_lossy().to_string(),
                file_type: "marketplace".to_string(),
            };
        }
    };

    let marketplace: PluginMarketplace = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            return ValidationResult {
                success: false,
                errors: vec![ValidationError {
                    path: "json".to_string(),
                    message: format!("Invalid JSON syntax: {}", e),
                }],
                warnings: Vec::new(),
                file_path: absolute_path.to_string_lossy().to_string(),
                file_type: "marketplace".to_string(),
            };
        }
    };

    let mut warnings = Vec::new();

    // Warn if no plugins
    if marketplace.plugins.is_empty() {
        warnings.push(ValidationWarning {
            path: "plugins".to_string(),
            message: "Marketplace has no plugins defined".to_string(),
        });
    }

    ValidationResult {
        success: true,
        errors: Vec::new(),
        warnings,
        file_path: absolute_path.to_string_lossy().to_string(),
        file_type: "marketplace".to_string(),
    }
}
