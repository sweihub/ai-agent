/**
 * Example 5: Custom System Prompt
 *
 * Shows how to customize the agent's behavior with a system prompt.
 *
 * Run: cargo run --example 05_custom_system_prompt
 */
use ai_agent::{Agent, AgentOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 5: Custom System Prompt ---\n");

    let mut agent = Agent::create(AgentOptions {
        model: Some(std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string())),
        max_turns: Some(5),
        system_prompt: Some(
            "You are a senior code reviewer. When asked to review code, focus on: \
             1) Security issues, 2) Performance concerns, 3) Maintainability. \
             Be concise and use bullet points.".to_string()
        ),
        ..Default::default()
    });

    let result = agent.prompt("Read src/agent.rs and give a brief code review.").await?;
    println!("{}", result.text);

    Ok(())
}