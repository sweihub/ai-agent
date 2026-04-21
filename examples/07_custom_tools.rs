/**
 * Example 7: Custom Tools
 *
 * Shows how to define and use custom tools alongside built-in tools.
 *
 * Run: cargo run --example 07_custom_tools
 */
use ai_agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 7: Custom Tools ---\n");

    // Note: Custom tools require AgentOptions with tools field
    // For this example, we'll just show available tools
    let builtin_tools = ai_agent::get_all_tools();

    let mut agent = Agent::new(
        &std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
        10,
    );

    println!("Loaded {} built-in tools\n", builtin_tools.len());

    let result = agent.query(
        "What is the weather in Tokyo and London? Also calculate 2**10 * 3. Be brief."
    ).await?;

    println!("Answer: {}", result.text);

    Ok(())
}