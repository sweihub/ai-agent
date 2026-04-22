/**
 * Example 23: Permission System
 *
 * Demonstrates the permission system:
 * - Permission modes (default, bypass, dontAsk, acceptEdits)
 * - Permission rules (allow, deny, ask)
 * - Permission context and handler
 * - Tool permission checking
 *
 * Run: cargo run --example 23_permissions
 */
use ai_agent::{
    PermissionBehavior, PermissionContext, PermissionHandler, PermissionMetadata, PermissionMode,
    PermissionRule,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 23: Permission System ---\n");

    // Example 1: Permission modes
    println!("=== 1. Permission Modes ===\n");

    // Default mode - asks for permission
    let ctx = PermissionContext::new().with_mode(PermissionMode::Default);
    let result = ctx.check_tool("Bash", None);
    println!(
        "Default mode: {:?}",
        result.message().unwrap_or(&"no message".to_string())
    );

    // Bypass mode - allows everything
    let ctx = PermissionContext::new().with_mode(PermissionMode::Bypass);
    let result = ctx.check_tool("Bash", None);
    println!(
        "Bypass mode: {}",
        if result.is_allowed() {
            "allowed"
        } else {
            "denied"
        }
    );

    // DontAsk mode - denies everything
    let ctx = PermissionContext::new().with_mode(PermissionMode::DontAsk);
    let result = ctx.check_tool("Bash", None);
    println!(
        "DontAsk mode: {}",
        if result.is_denied() {
            "denied"
        } else {
            "allowed"
        }
    );

    // AcceptEdits mode - allows edit tools
    let ctx = PermissionContext::new().with_mode(PermissionMode::AcceptEdits);
    let write_result = ctx.check_tool("Write", None);
    let read_result = ctx.check_tool("Read", None);
    println!(
        "AcceptEdits mode - Write: {}, Read: {}",
        if write_result.is_allowed() {
            "allowed"
        } else {
            "denied"
        },
        if read_result.is_allowed() {
            "allowed"
        } else {
            "denied"
        }
    );

    println!();

    // Example 2: Permission rules
    println!("=== 2. Permission Rules ===\n");

    // Allow rule - always allow a tool
    let ctx = PermissionContext::new().with_allow_rule(PermissionRule::allow("Read"));

    let result = ctx.check_tool("Read", None);
    println!(
        "Allow rule for Read: {}",
        if result.is_allowed() {
            "allowed"
        } else {
            "denied"
        }
    );

    // Deny rule - always deny a tool
    let ctx = PermissionContext::new().with_deny_rule(PermissionRule::deny("Bash"));

    let result = ctx.check_tool("Bash", None);
    println!(
        "Deny rule for Bash: {}",
        if result.is_denied() {
            "denied"
        } else {
            "allowed"
        }
    );

    // Ask rule - always ask
    let ctx = PermissionContext::new().with_ask_rule(PermissionRule::ask("Grep"));

    let result = ctx.check_tool("Grep", None);
    println!(
        "Ask rule for Grep: {}",
        if result.is_ask() { "asks" } else { "auto" }
    );

    println!();

    // Example 3: Permission handler with callback
    println!("=== 3. Permission Handler with Callback ===\n");

    let handler =
        PermissionHandler::new(PermissionContext::new().with_mode(PermissionMode::Default));

    // Check without callback
    let metadata = PermissionMetadata::new("Bash");
    let result = handler.check(metadata.clone());
    println!(
        "Without callback: {}",
        result.message().unwrap_or(&"allowed".to_string())
    );

    // Create handler with custom callback
    let handler = PermissionHandler::new(
        PermissionContext::new().with_allow_rule(PermissionRule::allow("Read")),
    );

    // Custom callback that overrides decisions
    let result = handler.check(metadata.clone());
    println!(
        "With rules: {}",
        if result.is_allowed() {
            "allowed"
        } else {
            "denied"
        }
    );

    println!();

    // Example 4: Rule with content matching
    println!("=== 4. Rules with Content Matching ===\n");

    // Allow rule with content pattern
    let ctx = PermissionContext::new().with_allow_rule(PermissionRule::with_content(
        "Bash",
        PermissionBehavior::Allow,
        "ls",
    ));

    let input1 = serde_json::json!({"command": "ls -la"});
    let result = ctx.check_tool("Bash", Some(&input1));
    println!(
        "Bash with 'ls': {}",
        if result.is_allowed() {
            "allowed"
        } else {
            "denied"
        }
    );

    let input2 = serde_json::json!({"command": "rm -rf /"});
    let result = ctx.check_tool("Bash", Some(&input2));
    println!(
        "Bash with 'rm': {}",
        if result.is_allowed() {
            "allowed"
        } else {
            "denied"
        }
    );

    println!();

    println!("=== done ===");
    Ok(())
}
