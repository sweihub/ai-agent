// Source: ~/claudecode/openclaudecode/src/utils/permissions/yoloClassifier.ts
#![allow(dead_code)]

//! YOLO (auto mode) classifier for security decisions.
//!
//! Uses an LLM to classify whether agent actions should be allowed or blocked.

use super::bash_classifier::{
    get_bash_prompt_allow_descriptions, get_bash_prompt_deny_descriptions,
};
use super::classifier_shared::{ContentBlock, extract_tool_use_block};
use crate::types::permissions::{ClassifierUsage, ToolPermissionContext, YoloClassifierResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// YOLO classifier tool name.
pub const YOLO_CLASSIFIER_TOOL_NAME: &str = "classify_result";

/// Transcript block for the classifier.
#[derive(Debug, Clone)]
pub enum TranscriptBlock {
    Text {
        text: String,
    },
    ToolUse {
        name: String,
        input: serde_json::Value,
    },
}

/// Transcript entry.
#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    pub role: String, // "user" or "assistant"
    pub content: Vec<TranscriptBlock>,
}

/// Builds transcript entries from messages.
pub fn build_transcript_entries(messages: &[serde_json::Value]) -> Vec<TranscriptEntry> {
    let mut transcript = Vec::new();

    for msg in messages {
        if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
            match msg_type {
                "user" => {
                    if let Some(content) = msg.get("message").and_then(|m| m.get("content")) {
                        let text_blocks = extract_text_blocks(content);
                        if !text_blocks.is_empty() {
                            transcript.push(TranscriptEntry {
                                role: "user".to_string(),
                                content: text_blocks
                                    .into_iter()
                                    .map(|t| TranscriptBlock::Text { text: t })
                                    .collect(),
                            });
                        }
                    }
                }
                "assistant" => {
                    if let Some(content) = msg.get("message").and_then(|m| m.get("content")) {
                        let blocks = extract_tool_use_blocks(content);
                        if !blocks.is_empty() {
                            transcript.push(TranscriptEntry {
                                role: "assistant".to_string(),
                                content: blocks,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    transcript
}

fn extract_text_blocks(content: &serde_json::Value) -> Vec<String> {
    let mut texts = Vec::new();
    if let Some(s) = content.as_str() {
        texts.push(s.to_string());
    } else if let Some(arr) = content.as_array() {
        for block in arr {
            if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                if block_type == "text" {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        texts.push(text.to_string());
                    }
                }
            }
        }
    }
    texts
}

fn extract_tool_use_blocks(content: &serde_json::Value) -> Vec<TranscriptBlock> {
    let mut blocks = Vec::new();
    if let Some(arr) = content.as_array() {
        for block in arr {
            if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                if block_type == "tool_use" {
                    if let (Some(name), Some(input)) = (
                        block.get("name").and_then(|v| v.as_str()),
                        block.get("input"),
                    ) {
                        blocks.push(TranscriptBlock::ToolUse {
                            name: name.to_string(),
                            input: input.clone(),
                        });
                    }
                }
            }
        }
    }
    blocks
}

/// Builds a compact transcript string for the classifier.
pub fn build_transcript_for_classifier(
    messages: &[serde_json::Value],
    _tools: &[serde_json::Value],
) -> String {
    let entries = build_transcript_entries(messages);
    let mut result = String::new();

    for entry in entries {
        for block in entry.content {
            match block {
                TranscriptBlock::Text { text } => {
                    result.push_str(&format!("User: {}\n", text));
                }
                TranscriptBlock::ToolUse { name, input } => {
                    let input_str = serde_json::to_string(&input).unwrap_or_default();
                    result.push_str(&format!("{} {}\n", name, input_str));
                }
            }
        }
    }

    result
}

/// Formats an action for the classifier.
pub fn format_action_for_classifier(
    tool_name: &str,
    tool_input: serde_json::Value,
) -> TranscriptEntry {
    TranscriptEntry {
        role: "assistant".to_string(),
        content: vec![TranscriptBlock::ToolUse {
            name: tool_name.to_string(),
            input: tool_input,
        }],
    }
}

/// YOLO classifier response schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloClassifierResponse {
    pub thinking: String,
    #[serde(rename = "shouldBlock")]
    pub should_block: bool,
    pub reason: String,
}

/// Builds the YOLO system prompt.
pub async fn build_yolo_system_prompt(context: &ToolPermissionContext) -> String {
    let base_prompt = get_base_prompt();
    let permissions_template = get_permissions_template(context);

    let auto_mode_allow = get_auto_mode_allow(context);
    let auto_mode_deny = get_auto_mode_deny(context);
    let auto_mode_env = get_auto_mode_environment(context);

    let mut system_prompt = base_prompt.replace("<permissions_template>", &permissions_template);

    if !auto_mode_allow.is_empty() {
        system_prompt = system_prompt.replace(
            "<user_allow_rules_to_replace></user_allow_rules_to_replace>",
            &auto_mode_allow,
        );
    }
    if !auto_mode_deny.is_empty() {
        system_prompt = system_prompt.replace(
            "<user_deny_rules_to_replace></user_deny_rules_to_replace>",
            &auto_mode_deny,
        );
    }
    if !auto_mode_env.is_empty() {
        system_prompt = system_prompt.replace(
            "<user_environment_to_replace></user_environment_to_replace>",
            &auto_mode_env,
        );
    }

    system_prompt
}

fn get_base_prompt() -> String {
    // In a full implementation, this would load from a bundled file
    "You are a security classifier for an AI agent. Evaluate whether actions should be allowed or blocked.".to_string()
}

fn get_permissions_template(context: &ToolPermissionContext) -> String {
    // Check if using external permissions
    if is_using_external_permissions() {
        get_external_permissions_template()
    } else {
        get_anthropic_permissions_template()
    }
}

fn is_using_external_permissions() -> bool {
    std::env::var("USER_TYPE").as_deref() != Ok("ant")
}

fn get_external_permissions_template() -> String {
    "## Allow Rules\n<user_allow_rules_to_replace></user_allow_rules_to_replace>\n\n## Deny Rules\n<user_deny_rules_to_replace></user_deny_rules_to_replace>\n\n## Environment\n<user_environment_to_replace></user_environment_to_replace>".to_string()
}

fn get_anthropic_permissions_template() -> String {
    "## Allow Rules\n<user_allow_rules_to_replace>- Read-only file access\n- Safe development commands</user_allow_rules_to_replace>\n\n## Deny Rules\n<user_deny_rules_to_replace>- Code execution without explicit permission\n- Network exfiltration attempts\n- Irreversible file deletion</user_deny_rules_to_replace>\n\n## Environment\n<user_environment_to_replace>- Development environment\n- Standard project structure</user_environment_to_replace>".to_string()
}

fn get_auto_mode_allow(context: &ToolPermissionContext) -> String {
    let mut allow = Vec::new();

    // Include bash prompt rules for ant builds
    if std::env::var("USER_TYPE").as_deref() == Ok("ant") && !is_using_external_permissions() {
        allow.extend(get_bash_prompt_allow_descriptions(&()));
    }

    allow.sort();
    allow.dedup();

    allow
        .iter()
        .map(|d| format!("- {}", d))
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_auto_mode_deny(context: &ToolPermissionContext) -> String {
    let mut deny = Vec::new();

    if std::env::var("USER_TYPE").as_deref() == Ok("ant") && !is_using_external_permissions() {
        deny.extend(get_bash_prompt_deny_descriptions(&()));
    }

    // PowerShell deny guidance
    if std::env::var("CLAUDE_CODE_POWERSHELL_AUTO_MODE").as_deref() == Ok("1")
        || std::env::var("CLAUDE_CODE_POWERSHELL_AUTO_MODE").as_deref() == Ok("true")
    {
        deny.extend(get_powershell_deny_guidance());
    }

    deny.sort();
    deny.dedup();

    deny.iter()
        .map(|d| format!("- {}", d))
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_auto_mode_environment(context: &ToolPermissionContext) -> String {
    let _ = context;
    String::new()
}

fn get_powershell_deny_guidance() -> Vec<String> {
    vec![
        "PowerShell Download-and-Execute: iex (iwr ...) and similar fall under \"Code from External\".".to_string(),
        "PowerShell Irreversible Destruction: Remove-Item -Recurse -Force falls under \"Irreversible Local Destruction\".".to_string(),
        "PowerShell Persistence: modifying $PROFILE or registry Run keys falls under \"Unauthorized Persistence\".".to_string(),
        "PowerShell Elevation: Start-Process -Verb RunAs falls under \"Security Weaken\".".to_string(),
    ]
}

/// Classifies a YOLO action.
pub async fn classify_yolo_action(
    _messages: &[serde_json::Value],
    _action: TranscriptEntry,
    _tools: &[serde_json::Value],
    _context: &ToolPermissionContext,
    _signal: &tokio::sync::oneshot::Receiver<()>,
) -> YoloClassifierResult {
    // In a full implementation, this would call the LLM API
    YoloClassifierResult {
        thinking: None,
        should_block: false,
        reason: "Classifier not available".to_string(),
        unavailable: Some(true),
        transcript_too_long: None,
        model: "unknown".to_string(),
        usage: None,
        duration_ms: None,
        prompt_lengths: None,
        error_dump_path: None,
        stage: None,
        stage1_usage: None,
        stage1_duration_ms: None,
        stage1_request_id: None,
        stage1_msg_id: None,
        stage2_usage: None,
        stage2_duration_ms: None,
        stage2_request_id: None,
        stage2_msg_id: None,
    }
}

/// Gets the auto mode dump directory.
pub fn get_auto_mode_dump_dir() -> String {
    let temp = std::env::temp_dir();
    temp.join("claude-auto-mode").to_string_lossy().to_string()
}

/// Gets the classifier error dump path.
pub fn get_auto_mode_classifier_error_dump_path() -> String {
    let temp = std::env::temp_dir();
    temp.join("auto-mode-classifier-errors")
        .to_string_lossy()
        .to_string()
}

/// Gets the classifier transcript.
pub fn get_auto_mode_classifier_transcript() -> Option<String> {
    // Would read from session state
    None
}

/// Checks if JSONL transcript format is enabled.
pub fn is_jsonl_transcript_enabled_yolo() -> bool {
    std::env::var("CLAUDE_CODE_JSONL_TRANSCRIPT")
        .ok()
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

/// Gets the default external auto mode rules.
pub fn get_default_external_auto_mode_rules() -> HashMap<String, Vec<String>> {
    let mut rules = HashMap::new();
    rules.insert("allow".to_string(), vec![]);
    rules.insert("soft_deny".to_string(), vec![]);
    rules.insert("environment".to_string(), vec![]);
    rules
}

/// Builds the default external system prompt.
pub fn build_default_external_system_prompt() -> String {
    let base = get_base_prompt();
    let template = get_external_permissions_template();
    base.replace("<permissions_template>", &template)
}
