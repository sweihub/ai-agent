/**
 * Example 12: ENV Configuration
 *
 * Demonstrates automatic loading of .env file configuration.
 * The SDK reads AI_BASE_URL, AI_AUTH_TOKEN, and AI_MODEL from:
 * 1. .env file in current directory
 * 2. .env file in parent directories
 * 3. Environment variables
 *
 * Create a .env file:
 * AI_BASE_URL="http://localhost:8000"
 * AI_AUTH_TOKEN="your-token"
 * AI_MODEL="claude-sonnet-4-6"
 *
 * Run: cargo run --example 12_env_config
 */
use ai_agent::{Agent, EnvConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 12: ENV Configuration ---\n");

    // Load env config (reads from .env file and environment)
    let config = EnvConfig::load();

    println!("Loaded configuration:");
    if let Some(url) = &config.base_url {
        println!("  AI_BASE_URL: {}", url);
    }
    if config.auth_token.is_some() {
        println!("  AI_AUTH_TOKEN: [hidden]");
    }
    if let Some(model) = &config.model {
        println!("  AI_MODEL: {}", model);
    }
    if config.extras.len() > 0 {
        println!("  Extra config:");
        for (key, value) in &config.extras {
            println!("    {}: {}", key, value);
        }
    }
    println!();

    // Create agent - will automatically use env config as defaults
    let mut agent = Agent::new(
        &std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
        5,
    );

    let result = agent.query("Say hello and confirm you received the configuration.").await?;

    println!("Answer: {}", result.text);

    Ok(())
}