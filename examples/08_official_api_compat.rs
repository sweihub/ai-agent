/**
 * Example 8: Official SDK-Compatible API
 *
 * Demonstrates the query() function with the same API pattern
 * as open-agent-sdk. Drop-in compatible.
 *
 * Run: cargo run --example 08_official_api_compat
 */
use ai_agent::{Agent, AgentOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 8: Official SDK-Compatible API ---\n");

    let mut agent = Agent::create(AgentOptions {
        model: Some(std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string())),
        allowed_tools: vec!["Bash".to_string(), "Glob".to_string()],
        ..Default::default()
    });

    // Using query with streaming-like behavior (for now just prompt)
    let result = agent.prompt("What files are in this directory? Be brief.").await?;

    println!("{}", result.text);
    println!("\nDone: success");

    Ok(())
}