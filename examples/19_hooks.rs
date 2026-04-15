/**
 * Example 19: Hooks System
 *
 * Demonstrates using hooks to intercept and modify tool behavior.
 *
 * Run: cargo run --example 19_hooks
 *
 * This example shows:
 * 1. Creating a HookRegistry
 * 2. Registering hooks for PreToolUse event
 * 3. Executing hooks and handling the output
 * 4. Using matchers to filter hooks by tool name
 */
use ai_agent::hooks::{HookRegistry, HookDefinition, HookInput, HOOK_EVENTS};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 19: Hooks System ---\n");

    // List all available hook events
    println!("Available hook events:");
    for event in HOOK_EVENTS {
        println!("  - {}", event);
    }

    // Create a hook registry
    println!("\n--- Creating Hook Registry ---");
    let mut registry = HookRegistry::new();
    println!("Created hook registry");

    // Register a PreToolUse hook with a shell command
    println!("\n--- Registering PreToolUse Hook ---");
    registry.register(
        "PreToolUse",
        HookDefinition {
            command: Some("echo PreToolUse: $(date +%T)".to_string()),
            timeout: Some(5000),
            matcher: Some("Read.*".to_string()), // Only trigger for Read tool
        },
    );
    println!("Registered PreToolUse hook for Read tool");

    // Register another hook for all tools (no matcher)
    registry.register(
        "PreToolUse",
        HookDefinition {
            command: Some("echo Any tool is about to run".to_string()),
            timeout: Some(3000),
            matcher: None, // Match all tools
        },
    );
    println!("Registered PreToolUse hook for all tools");

    // Check if hooks are registered
    println!("\n--- Checking Hooks ---");
    println!("Has PreToolUse hooks: {}", registry.has_hooks("PreToolUse"));
    println!("Has PostToolUse hooks: {}", registry.has_hooks("PostToolUse"));

    // Execute hooks for PreToolUse event
    println!("\n--- Executing Hooks ---");

    // Create input with tool info
    let mut input = HookInput::new("PreToolUse");
    input.tool_name = Some("Read".to_string());
    input.tool_input = Some(serde_json::json!({
        "file_path": "/tmp/test.txt",
        "offset": 1,
        "limit": 100
    }));
    input.tool_use_id = Some("toolu_12345".to_string());
    input.cwd = Some("/home/user".to_string());

    println!("Executing PreToolUse hooks for Read tool...");
    println!("  Tool: {}", input.tool_name.as_ref().unwrap());
    println!("  Input: {:?}", input.tool_input);

    let results = registry.execute("PreToolUse", input).await;

    println!("\nHook execution results:");
    for (i, result) in results.iter().enumerate() {
        println!("  Hook {}:", i + 1);
        if let Some(msg) = &result.message {
            println!("    Message: {}", msg);
        }
        if let Some(perm) = &result.permission_update {
            println!("    Permission: {:?}", perm);
        }
    }

    // Execute hooks for a non-matching tool (Bash)
    println!("\n--- Executing Hooks for Bash Tool ---");
    let mut input_bash = HookInput::new("PreToolUse");
    input_bash.tool_name = Some("Bash".to_string());
    input_bash.tool_input = Some(serde_json::json!({
        "command": "ls -la"
    }));

    let results_bash = registry.execute("PreToolUse", input_bash).await;
    println!("Results for Bash tool: {} hooks matched", results_bash.len());

    // Demonstrate PostToolUse hook
    println!("\n--- PostToolUse Hook ---");
    registry.register(
        "PostToolUse",
        HookDefinition {
            command: Some("echo PostToolUse completed".to_string()),
            timeout: Some(5000),
            matcher: None,
        },
    );

    let mut post_input = HookInput::new("PostToolUse");
    post_input.tool_name = Some("Glob".to_string());
    post_input.tool_output = Some(serde_json::json!({
        "files": ["a.rs", "b.rs", "c.rs"]
    }));

    let post_results = registry.execute("PostToolUse", post_input).await;
    println!("PostToolUse results: {} hooks executed", post_results.len());

    // Demonstrate SessionStart hook
    println!("\n--- SessionStart Hook ---");
    registry.register(
        "SessionStart",
        HookDefinition {
            command: Some("echo Session started at $(date)".to_string()),
            timeout: Some(3000),
            matcher: None,
        },
    );

    let session_input = HookInput::new("SessionStart");
    let session_results = registry.execute("SessionStart", session_input).await;
    println!("SessionStart results: {} hooks executed", session_results.len());

    println!("\n--- done ---");
    Ok(())
}