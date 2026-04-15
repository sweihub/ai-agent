/**
 * Example 15: Memory System Demo
 *
 * Demonstrates the persistent file-based memory system.
 * Shows how memories are stored and loaded across sessions.
 *
 * Run: cargo run --example 15_memory
 *
 * Environment variables from .env:
 * - AI_BASE_URL: LLM server URL
 * - AI_AUTH_TOKEN: API authentication token
 * - AI_MODEL: Model name
 * - AI_MEMORY_PATH_OVERRIDE: Override memory directory path
 * - AI_DISABLE_AUTO_MEMORY: Set to "1" to disable memory
 */
use ai_agent::{
    memdir::{ensure_memory_dir_exists, get_auto_mem_path, get_memory_entrypoint},
    Agent,
};

use std::fs;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 15: Memory System Demo ---\n");

    // Get memory directory path
    let memory_dir = get_auto_mem_path();
    let entrypoint = get_memory_entrypoint();

    println!("Memory directory: {}", memory_dir.display());
    println!("MEMORY.md: {}\n", entrypoint.display());

    // Ensure memory directory exists
    ensure_memory_dir_exists(&memory_dir)?;
    println!("Created memory directory");

    // Write example memory files
    write_example_memories(&memory_dir)?;

    // Update MEMORY.md index
    update_memory_index(&memory_dir, &entrypoint)?;

    // Show the memory prompt that gets injected
    println!("\n--- Memory Prompt Content ---\n");

    // Now create an agent - it will automatically load the memory
    let mut agent = Agent::new(
        &std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
        5,
    );

    // Ask about user preferences stored in memory
    let result = agent
        .prompt("What do you know about me? Check your memory.")
        .await?;

    println!("\n--- Agent Response ---\n");
    println!(
        "{}",
        result
            .text
            .trim()
            .lines()
            .take(10)
            .collect::<Vec<_>>()
            .join("\n")
    );

    println!("\n=== done ===");
    Ok(())
}

fn write_example_memories(memory_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // Write a user memory
    let user_memory = r#"---
name: user_expertise
description: User is a senior Rust developer with experience in systems programming
type: user
---

The user is a senior software engineer who prefers concise responses.
They are working on porting an AI agent SDK from TypeScript to Rust.
They have deep Go expertise and are learning Rust best practices.
"#;

    let user_path = memory_dir.join("user_expertise.md");
    fs::write(&user_path, user_memory)?;
    println!("Created: {}", user_path.display());

    // Write a feedback memory
    let feedback_memory = r#"---
name: feedback_testing
description: Prefers integration tests over mocked tests
type: feedback
---

Always prefer integration tests that hit real databases over mocked tests.
Why: We got burned last quarter when mocked tests passed but the prod migration failed.
How to apply: When writing tests for data layer code, use real DB connections if possible.
"#;

    let feedback_path = memory_dir.join("feedback_testing.md");
    fs::write(&feedback_path, feedback_memory)?;
    println!("Created: {}", feedback_path.display());

    // Write a project memory
    let project_memory = r#"---
name: project_context
description: AI Agent SDK port from TypeScript to Rust
type: project
---

Porting claude code CLI from TypeScript to Rust. Current focus on:
- Matching original project structure and behavior exactly
- Translating all test cases
- Maintaining OpenAI-compatible API format

Why: Need a native Rust SDK for embedded use cases.
"#;

    let project_path = memory_dir.join("project_context.md");
    fs::write(&project_path, project_memory)?;
    println!("Created: {}", project_path.display());

    // Write a reference memory
    let ref_memory = r#"---
name: reference_docs
description: Links to project documentation
type: reference
---

Original TypeScript project: ~/claudecode/openclaudecode
API documentation: https://developers.openai.com/api/reference/overview
"#;

    let ref_path = memory_dir.join("reference_docs.md");
    fs::write(&ref_path, ref_memory)?;
    println!("Created: {}", ref_path.display());

    Ok(())
}

fn update_memory_index(
    _memory_dir: &std::path::Path,
    entrypoint: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let index_content = r#"# auto memory

- [user_expertise](user_expertise.md) — User is a senior Rust developer with systems programming experience
- [feedback_testing](feedback_testing.md) — Prefers integration tests over mocked tests
- [project_context](project_context.md) — AI Agent SDK port from TypeScript to Rust
- [reference_docs](reference_docs.md) — Links to project documentation
"#;

    let mut file = fs::File::create(entrypoint)?;
    file.write_all(index_content.as_bytes())?;
    println!("Created: {}", entrypoint.display());

    Ok(())
}
