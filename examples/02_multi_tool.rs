/**
 * Example 2: Multi-Tool Orchestration
 *
 * The agent autonomously uses Glob, Bash, and Read tools to
 * accomplish a multi-step task.
 *
 * Run: cargo run --example 02_multi_tool
 */
use ai_agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 2: Multi-Tool Orchestration ---\n");

    let agent =
        Agent::new(&std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string()))
            .max_turns(15);

    let result = agent
        .query(
            "Do these steps: \
         1) Use Glob to find all .rs files in src/ (pattern \"src/**/*.rs\"). \
         2) Use Bash to count lines in src/lib.rs with `wc -l`. \
         3) Give a brief summary.",
        )
        .await?;

    println!("Answer: {}", result.text);
    println!(
        "Turns: {} | Tokens: {}/{}",
        result.num_turns, result.usage.input_tokens, result.usage.output_tokens
    );

    Ok(())
}
