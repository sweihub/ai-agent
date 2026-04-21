// Example: Query Stream API for CLI/TUI integration
//
// Demonstrates using agent.query_stream() to consume events incrementally
// with tokio::select! for responsive interrupt handling.

use ai_agent::Agent;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent with event callback for the traditional API
    let mut agent = Agent::new("MiniMaxAI/MiniMax-M2.5", 5);

    println!("=== Query Stream Example ===\n");
    println!("Running a query with incremental event streaming...\n");

    // query_stream returns a futures::Stream<Item = AgentEvent>
    // The engine loop runs on a spawned tokio task
    match agent.query_stream("List 3 interesting facts about Rust programming").await {
        Ok(stream) => {
            tokio::pin!(stream);

            loop {
                tokio::select! {
                    event = stream.next() => {
                        match event {
                            Some(ai_agent::types::AgentEvent::ContentBlockDelta {
                                delta: ai_agent::types::ContentDelta::Text { text },
                                ..
                            }) => {
                                print!("{}", text);
                                std::io::Write::flush(&mut std::io::stdout())?;
                            }
                            Some(ai_agent::types::AgentEvent::ContentBlockDelta {
                                delta: ai_agent::types::ContentDelta::Thinking { text },
                                ..
                            }) => {
                                eprintln!("[thinking: {}]", text.chars().take(50).collect::<String>());
                            }
                            Some(ai_agent::types::AgentEvent::ToolStart { tool_name, .. }) => {
                                eprintln!("\n[Tool: {}]", tool_name);
                            }
                            Some(ai_agent::types::AgentEvent::Thinking { turn }) => {
                                eprintln!("\n[Turn {}: thinking...]", turn);
                            }
                            Some(ai_agent::types::AgentEvent::Done { result }) => {
                                println!("\n\n=== Query Complete ===");
                                println!("Turns: {}", result.num_turns);
                                println!("Input tokens: {}", result.usage.input_tokens);
                                println!("Output tokens: {}", result.usage.output_tokens);
                                println!("Duration: {}ms", result.duration_ms);
                                println!("Exit reason: {:?}", result.exit_reason);
                                break;
                            }
                            Some(_) => {}
                            None => break,
                        }
                    }
                    // Simulate a timeout (in real TUI, this would be Ctrl+C handler)
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        eprintln!("\n[Timeout — no more events]");
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Query stream error: {}", e);
        }
    }

    // Alternative: subscribe() for pub/sub event consumption
    println!("\n\n=== Subscribe Example ===\n");

    let mut agent2 = Agent::new("MiniMaxAI/MiniMax-M2.5", 3);
    let (_sub, _guard) = agent2.subscribe();
    // Note: subscribe() returns an EventSubscriber Stream.
    // The above demonstrates the API shape. For a full subscribe example:
    //   let (mut sub, guard) = agent2.subscribe();
    //   while let Some(ev) = sub.next().await { ... }

    // Simple query via subscribe-compatible pattern
    match agent2.query("Tell me a joke").await {
        Ok(result) => {
            println!("Query complete via subscribe agent! Turns: {}", result.num_turns);
        }
        Err(e) => {
            eprintln!("Query error: {}", e);
        }
    }

    println!("\n\nDone!");
    Ok(())
}
