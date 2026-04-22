/**
 * Example 21: AI.md in Agent Context
 *
 * Demonstrates how AI.md instructions affect agent behavior.
 *
 * Run: cargo run --example 21_ai_md_agent
 *
 * This example shows:
 * 1. Loading AI.md from current directory
 * 2. Agent following the instructions from AI.md
 */
use ai_agent::{Agent, AiMdType, get_ai_md_files, load_ai_md};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 21: AI.md in Agent Context ---\n");

    let cwd = std::env::current_dir()?;

    // Load AI.md files
    println!("--- Loading AI.md files ---");
    let files = get_ai_md_files(&cwd)?;
    println!("Found {} file(s)\n", files.len());

    for f in &files {
        let type_str = match f.md_type {
            AiMdType::Managed => "Managed",
            AiMdType::User => "User",
            AiMdType::Project => "Project",
            AiMdType::Local => "Local",
        };
        println!("  [{}] {}", type_str, f.path);
        // Show content preview
        let preview = f.content.lines().take(5).collect::<Vec<_>>().join("\n");
        println!("  Content preview:\n{}", preview);
        println!();
    }

    // Load AI.md into prompt
    println!("\n--- Loading AI.md into context ---");
    let ai_md_prompt = load_ai_md(&cwd)?;
    match ai_md_prompt {
        Some(p) => {
            println!("AI.md prompt: {} chars\n", p.len());
            println!("AI.md content preview:");
            println!("{}", p.lines().take(15).collect::<Vec<_>>().join("\n"));
        }
        None => println!("No AI.md found"),
    }

    // Create agent and run with AI.md injected
    println!("\n--- Creating Agent with AI.md ---");
    let agent = Agent::new("MiniMaxAI/MiniMax-M2.5").max_turns(3);

    // Ask about the project - agent should follow AI.md instructions
    let result = agent
        .query("Tell me about this project in one sentence.")
        .await?;

    println!("\n--- Agent Response ---");
    println!("{}", result.text);

    println!("\n--- done ---");
    Ok(())
}
