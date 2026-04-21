/**
 * Example 4: Simple Prompt API
 *
 * Uses the blocking prompt() method for quick one-shot queries.
 * No need to iterate over streaming events.
 *
 * Run: cargo run --example 04_prompt_api
 */
use ai_agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 4: Simple Prompt API ---\n");

    let mut agent = Agent::new(
        &std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
        5,
    );

    let result = agent.query(
        "Use Bash to run `rustc --version` and `cargo --version`, then tell me the versions."
    ).await?;

    println!("Answer: {}", result.text);
    println!("Turns: {}", result.num_turns);
    println!("Tokens: {} in / {} out", result.usage.input_tokens, result.usage.output_tokens);
    println!("Duration: {}ms", result.duration_ms);

    Ok(())
}