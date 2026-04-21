/**
 * Example 10: Permissions and Allowed Tools
 *
 * Shows how to restrict which tools the agent can use.
 * Creates a read-only agent that can analyze but not modify code.
 *
 * Run: cargo run --example 10_permissions
 */
use ai_agent::{Agent, AgentOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 10: Read-Only Agent ---\n");

    // Read-only agent: can only use Read, Glob, Grep
    let mut agent = Agent::create(AgentOptions {
        model: Some(std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string())),
        allowed_tools: vec![
            "Read".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
        ],
        ..Default::default()
    });

    let result = agent.query(
        "Review the code in src/agent.rs for best practices. Be concise."
    ).await?;

    println!("{}", result.text);
    println!("\n--- done ---");

    Ok(())
}