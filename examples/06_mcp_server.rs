/**
 * Example 6: MCP Server Integration
 *
 * Connects to an MCP (Model Context Protocol) server and uses
 * its tools through the agent. This example uses the filesystem
 * MCP server as a demonstration.
 *
 * Prerequisites:
 *   npm install -g @modelcontextprotocol/server-filesystem
 *
 * Run: cargo run --example 06_mcp_server
 */
use ai_agent::{Agent, McpServerConfig, McpStdioConfig};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 6: MCP Server Integration ---\n");

    let agent =
        Agent::new(&std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string()))
            .max_turns(10)
            .mcp_servers(HashMap::from([(
                "filesystem".to_string(),
                McpServerConfig::Stdio(McpStdioConfig {
                    transport_type: Some("stdio".to_string()),
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                        "/tmp".to_string(),
                    ]),
                    env: None,
                }),
            )]));

    println!("Connecting to MCP filesystem server...\n");

    let result = agent
        .query("Use the filesystem MCP tools to list files in /tmp. Be brief.")
        .await?;

    println!("Answer: {}", result.text);
    println!("Turns: {}", result.num_turns);

    Ok(())
}
