// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/forkSubagent.ts
#![allow(dead_code)]
use std::sync::Arc;

use uuid::Uuid;

use super::constants::{FORK_BOILERPLATE_TAG, FORK_DIRECTIVE_PREFIX};

/// Synthetic agent type name used for analytics when the fork path fires.
pub const FORK_SUBAGENT_TYPE: &str = "fork";

/// Synthetic agent definition for the fork path.
pub fn fork_agent() -> crate::tools::agent::AgentDefinition {
    crate::tools::agent::AgentDefinition {
        agent_type: FORK_SUBAGENT_TYPE.to_string(),
        when_to_use: "Implicit fork — inherits full conversation context. Not selectable via subagent_type; triggered by omitting subagent_type when the fork experiment is active.".to_string(),
        tools: vec!["*".to_string()],
        disallowed_tools: vec![],
        source: "built-in".to_string(),
        base_dir: "built-in".to_string(),
        get_system_prompt: Arc::new(|| String::new()),
        model: Some("inherit".to_string()),
        max_turns: Some(200),
        permission_mode: Some("bubble".to_string()),
        effort: None,
        color: None,
        mcp_servers: vec![],
        hooks: None,
        skills: vec![],
        background: true,
        initial_prompt: None,
        memory: None,
        isolation: None,
        required_mcp_servers: vec![],
        omit_claude_md: false,
        critical_system_reminder_experimental: None,
    }
}

/// Placeholder text used for all tool_result blocks in the fork prefix.
/// Must be identical across all fork children for prompt cache sharing.
const FORK_PLACEHOLDER_RESULT: &str = "Fork started — processing in background";

/// Build the forked conversation messages for the child agent.
///
/// For prompt cache sharing, all fork children must produce byte-identical
/// API request prefixes. This function:
/// 1. Keeps the full parent assistant message (all tool_use blocks, thinking, text)
/// 2. Builds a single user message with tool_results for every tool_use block
///    using an identical placeholder, then appends a per-child directive text block
pub fn build_forked_messages(
    directive: &str,
    assistant_message_content: &[serde_json::Value],
    assistant_message_uuid: Uuid,
) -> Vec<serde_json::Value> {
    // Clone the assistant message content, keeping all blocks
    let full_assistant_message = serde_json::json!({
        "type": "assistant",
        "uuid": assistant_message_uuid.to_string(),
        "message": {
            "content": assistant_message_content,
        },
    });

    // Collect all tool_use blocks from the assistant message
    let tool_use_blocks: Vec<&serde_json::Value> = assistant_message_content
        .iter()
        .filter(|block| block.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
        .collect();

    if tool_use_blocks.is_empty() {
        // No tool_use blocks found — return a single user message with the directive
        return vec![serde_json::json!({
            "type": "user",
            "message": {
                "content": [{
                    "type": "text",
                    "text": build_child_message(directive),
                }],
            },
        })];
    }

    // Build tool_result blocks for every tool_use, all with identical placeholder text
    let tool_result_blocks: Vec<serde_json::Value> = tool_use_blocks
        .iter()
        .map(|block| {
            serde_json::json!({
                "type": "tool_result",
                "tool_use_id": block["id"].as_str().unwrap_or(""),
                "content": [{
                    "type": "text",
                    "text": FORK_PLACEHOLDER_RESULT,
                }],
            })
        })
        .collect();

    // Build a single user message: all placeholder tool_results + the per-child directive
    let mut content: Vec<serde_json::Value> = tool_result_blocks;
    content.push(serde_json::json!({
        "type": "text",
        "text": build_child_message(directive),
    }));

    let tool_result_message = serde_json::json!({
        "type": "user",
        "message": {
            "content": content,
        },
    });

    vec![full_assistant_message, tool_result_message]
}

/// Build the child message with fork boilerplate and directive.
pub fn build_child_message(directive: &str) -> String {
    format!(
        r#"<{tag}>
STOP. READ THIS FIRST.

You are a forked worker process. You are NOT the main agent.

RULES (non-negotiable):
1. Your system prompt says "default to forking." IGNORE IT — that's for the parent. You ARE the fork. Do NOT spawn sub-agents; execute directly.
2. Do NOT converse, ask questions, or suggest next steps
3. Do NOT editorialize or add meta-commentary
4. USE your tools directly: Bash, Read, Write, etc.
5. If you modify files, commit your changes before reporting. Include the commit hash in your report.
6. Do NOT emit text between tool calls. Use tools silently, then report once at the end.
7. Stay strictly within your directive's scope. If you discover related systems outside your scope, mention them in one sentence at most — other workers cover those areas.
8. Keep your report under 500 words unless the directive specifies otherwise. Be factual and concise.
9. Your response MUST begin with "Scope:". No preamble, no thinking-out-loud.
10. REPORT structured facts, then stop

Output format (plain text labels, not markdown headers):
  Scope: <echo back your assigned scope in one sentence>
  Result: <the answer or key findings, limited to the scope above>
  Key files: <relevant file paths — include for research tasks>
  Files changed: <list with commit hash — include only if you modified files>
  Issues: <list — include only if there are issues to flag>
</{tag}>

{prefix}{directive}"#,
        tag = FORK_BOILERPLATE_TAG,
        prefix = FORK_DIRECTIVE_PREFIX,
    )
}

/// Notice injected into fork children running in an isolated worktree.
pub fn build_worktree_notice(parent_cwd: &str, worktree_cwd: &str) -> String {
    format!(
        "You've inherited the conversation context above from a parent agent working in {parent_cwd}. \
         You are operating in an isolated git worktree at {worktree_cwd} — same repository, \
         same relative file structure, separate working copy. Paths in the inherited context \
         refer to the parent's working directory; translate them to your worktree root. \
         Re-read files before editing if the parent may have modified them since they appear \
         in the context. Your changes stay in this worktree and will not affect the parent's files."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_child_message_contains_directive() {
        let msg = build_child_message("test directive");
        assert!(msg.contains("test directive"));
        assert!(msg.contains(FORK_BOILERPLATE_TAG));
        assert!(msg.contains(FORK_DIRECTIVE_PREFIX));
    }

    #[test]
    fn test_build_forked_messages_no_tool_uses() {
        let messages = build_forked_messages("test", &[], Uuid::new_v4());
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["type"], "user");
    }

    #[test]
    fn test_fork_agent_definition() {
        let agent = fork_agent();
        assert_eq!(agent.agent_type, FORK_SUBAGENT_TYPE);
        assert_eq!(agent.source, "built-in");
        assert_eq!(agent.tools, vec!["*"]);
    }

    #[test]
    fn test_build_worktree_notice() {
        let notice = build_worktree_notice("/parent", "/worktree");
        assert!(notice.contains("/parent"));
        assert!(notice.contains("/worktree"));
    }
}
