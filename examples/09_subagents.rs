/**
 * Example 9: Subagents
 *
 * Define specialized subagents that the main agent can delegate
 * tasks to. Uses the Agent tool for subagent delegation.
 *
 * Run: cargo run --example 09_subagents
 *
 * Environment variables from .env:
 * - AI_BASE_URL: LLM server URL
 * - AI_AUTH_TOKEN: API authentication token
 * - AI_MODEL: Model name (defaults to MiniMaxAI/MiniMax-M2.5)
 */
use ai_agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 9: Subagents ---\n");

    // Get model from environment or use default
    let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "MiniMaxAI/MiniMax-M2.5".to_string());

    println!("Using model: {}\n", model);

    let mut agent = Agent::new(&model, 10);

    // The main agent can still use subagents via the Agent tool
    let result = agent.prompt("Review the code in src/agent.rs for best practices. Be concise.").await?;

    println!("{}", result.text);
    println!("\n--- done ---");

    Ok(())
}