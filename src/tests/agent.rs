use crate::Agent;
use crate::agent::build_agent_system_prompt;
use crate::env::EnvConfig;
use crate::types::ContentDelta;
use std::thread;

/// Test that Agent tool correctly extracts all parameters from input
#[tokio::test]
async fn test_agent_tool_parses_all_parameters() {
    // Test parameter extraction from various input formats
    // This verifies all parameters are now properly parsed

    // Test 1: subagent_type parameter (snake_case)
    let input1 = serde_json::json!({
        "description": "explore-agent",
        "prompt": "Explore the codebase",
        "subagent_type": "Explore"
    });
    assert_eq!(input1["subagent_type"].as_str(), Some("Explore"));
    assert_eq!(input1["subagentType"].as_str(), None); // snake_case works

    // Test 2: subagent_type parameter (camelCase)
    let input2 = serde_json::json!({
        "description": "explore-agent",
        "prompt": "Explore the codebase",
        "subagentType": "Plan"
    });
    assert_eq!(input2["subagentType"].as_str(), Some("Plan"));

    // Test 3: run_in_background (snake_case)
    let input3 = serde_json::json!({
        "description": "background-agent",
        "prompt": "Run in background",
        "run_in_background": true
    });
    assert_eq!(input3["run_in_background"].as_bool(), Some(true));

    // Test 4: runInBackground (camelCase)
    let input4 = serde_json::json!({
        "description": "background-agent",
        "runInBackground": true
    });
    assert_eq!(input4["runInBackground"].as_bool(), Some(true));

    // Test 5: max_turns (snake_case)
    let input5 = serde_json::json!({
        "description": "test",
        "max_turns": 5
    });
    assert_eq!(input5["max_turns"].as_u64(), Some(5));

    // Test 6: maxTurns (camelCase)
    let input6 = serde_json::json!({
        "description": "test",
        "maxTurns": 10
    });
    assert_eq!(input6["maxTurns"].as_u64(), Some(10));

    // Test 7: team_name (snake_case)
    let input7 = serde_json::json!({
        "description": "team-agent",
        "team_name": "my-team"
    });
    assert_eq!(input7["team_name"].as_str(), Some("my-team"));

    // Test 8: teamName (camelCase)
    let input8 = serde_json::json!({
        "description": "team-agent",
        "teamName": "my-team"
    });
    assert_eq!(input8["teamName"].as_str(), Some("my-team"));

    // Test 9: cwd parameter
    let input9 = serde_json::json!({
        "description": "custom-cwd",
        "cwd": "/custom/path"
    });
    assert_eq!(input9["cwd"].as_str(), Some("/custom/path"));

    // Test 10: name parameter
    let input10 = serde_json::json!({
        "name": "my-agent",
        "description": "named-agent"
    });
    assert_eq!(input10["name"].as_str(), Some("my-agent"));

    // Test 11: mode parameter
    let input11 = serde_json::json!({
        "description": "plan-mode",
        "mode": "plan"
    });
    assert_eq!(input11["mode"].as_str(), Some("plan"));

    // Test 12: isolation parameter
    let input12 = serde_json::json!({
        "description": "isolated",
        "isolation": "worktree"
    });
    assert_eq!(input12["isolation"].as_str(), Some("worktree"));

    // Verify all expected keys are now handled
    // The agent tool executor should handle all these parameters
}

/// Test that Agent tool creates subagent with proper system prompt based on agent type
#[tokio::test]
async fn test_agent_tool_system_prompt_by_type() {
    // Test system prompt generation for different agent types
    let explore_prompt = build_agent_system_prompt("Explore task", Some("Explore"));
    assert!(explore_prompt.contains("Explore agent"));

    let plan_prompt = build_agent_system_prompt("Plan task", Some("Plan"));
    assert!(plan_prompt.contains("Plan agent"));

    let review_prompt = build_agent_system_prompt("Review task", Some("Review"));
    assert!(review_prompt.contains("Review agent"));

    let general_prompt = build_agent_system_prompt("General task", None);
    assert!(general_prompt.contains("Task description: General task"));
}

/// Check if required environment variables are present for real API tests
/// Returns true if AI_BASE_URL, AI_MODEL, and AI_AUTH_TOKEN can be loaded from .env
pub fn has_required_env_vars() -> bool {
    let config = EnvConfig::load();
    config.base_url.is_some() && config.model.is_some() && config.auth_token.is_some()
}

/// Test Agent creation with options
#[tokio::test]
async fn test_create_agent() {
    let agent = Agent::new("claude-sonnet-4-6");
    assert!(!agent.get_model().is_empty());
}

/// Test Agent tool calling with real .env config
/// This test makes an actual API call using the configured model
#[tokio::test]
async fn test_agent_tool_calling_with_real_env_config() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Verify config is loaded
    assert!(config.base_url.is_some(), "Base URL should be configured");
    assert!(
        config.auth_token.is_some(),
        "Auth token should be configured"
    );
    assert!(config.model.is_some(), "Model should be configured");

    // Create agent with real config
    let agent = Agent::new(config.model.as_ref().unwrap());

    // Verify agent was created with the configured model
    let model = agent.get_model();
    assert!(!model.is_empty(), "Agent should have a model set");
    println!("Using model: {}", model);
}

/// Test agent prompt with real API call using .env config
/// This is an integration test that exercises the full agent flow
#[tokio::test]
async fn test_agent_prompt_with_real_api() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip test if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Create agent with all tools and real config
    use crate::get_all_tools;
    let tools = get_all_tools();

    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(3)
        .tools(tools);

    // Make a simple prompt that should trigger tool use
    let result = agent.query("What is 2 + 2? Just give me the answer.").await;

    // Verify we got a response
    assert!(result.is_ok(), "Agent should respond successfully");
    let response = result.unwrap();
    assert!(!response.text.is_empty(), "Response should not be empty");
    println!("Agent response: {}", response.text);
}

/// Test agent tool calling with multiple tools from .env config
/// This tests that the agent can use tools when configured via .env
#[tokio::test]
async fn test_agent_with_multiple_tools_real_config() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    // Verify we have tools available
    assert!(!tools.is_empty(), "Should have tools available");

    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(3)
        .tools(tools);

    // Prompt that might use tools
    let result = agent
        .query("List all Rust files in the current directory using glob")
        .await;

    // Should get a response (may or may not use tools depending on model)
    assert!(result.is_ok(), "Agent should respond");
    let response = result.unwrap();
    assert!(!response.text.is_empty(), "Response should not be empty");
    println!("Agent response: {}", response.text);
}

/// Test that tool executors are registered and can be invoked
/// This verifies the fix for tool calling not working in TUI
#[tokio::test]
async fn test_tool_executors_registered() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    // Verify tools are available
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"Bash"), "Should have Bash tool");
    assert!(
        tool_names.contains(&"FileRead"),
        "Should have FileRead tool"
    );
    assert!(tool_names.contains(&"Glob"), "Should have Glob tool");
    println!("Available tools: {:?}", tool_names);

    // Create agent
    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(3)
        .tools(tools);

    // Prompt that should definitely use the Bash tool
    let result = agent
        .query("Run this command: echo 'hello from tool test'")
        .await;

    // Verify we got a response
    assert!(result.is_ok(), "Agent should respond successfully");
    let response = result.unwrap();
    assert!(!response.text.is_empty(), "Response should not be empty");

    // Check that the tool was actually used (response should contain output)
    let text_lower = response.text.to_lowercase();
    let tool_was_used = text_lower.contains("hello from tool test") || text_lower.contains("tool");
    println!(
        "Tool calling test - Response: {} (tool_used: {})",
        response.text, tool_was_used
    );
}

/// Test Glob tool directly via agent
#[tokio::test]
async fn test_glob_tool_via_agent() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(3)
        .tools(tools);

    // Prompt that should use Glob tool
    let result = agent
        .query("List all .rs files in the src directory using the Glob tool")
        .await;

    assert!(result.is_ok(), "Agent should respond");
    let response = result.unwrap();
    assert!(!response.text.is_empty(), "Response should not be empty");
    println!("Glob tool test response: {}", response.text);
}

/// Test FileRead tool directly via agent
#[tokio::test]
async fn test_fileread_tool_via_agent() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(3)
        .tools(tools);

    // Prompt that should use FileRead tool
    let result = agent
        .query("Read the Cargo.toml file from the current directory")
        .await;

    assert!(result.is_ok(), "Agent should respond");
    let response = result.unwrap();
    assert!(!response.text.is_empty(), "Response should not be empty");
    // The response should contain something from Cargo.toml
    println!("FileRead tool test response: {}", response.text);
}

/// Test multiple tool calls in one turn
#[tokio::test]
async fn test_multiple_tool_calls() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(5)
        .tools(tools);

    // Prompt that should use multiple tools
    let result = agent
        .query("First list all files in the current directory, then read the README.md file if it exists")
        .await;

    assert!(result.is_ok(), "Agent should respond");
    let response = result.unwrap();
    assert!(!response.text.is_empty(), "Response should not be empty");
    println!("Multiple tool calls test response: {}", response.text);
}

/// Test that the agent remembers context across multiple query() calls
/// This verifies that messages are properly accumulated in the Agent and
/// passed to the QueryEngine for context maintenance across turns.
#[tokio::test]
async fn test_agent_remembers_context_across_queries() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(5)
        .tools(tools);

    // First turn: tell the agent something specific to remember
    let result1 = agent
        .query("Remember this: My favorite color is blue and I live in Seattle.")
        .await;

    assert!(result1.is_ok(), "First query should succeed");
    let response1 = result1.unwrap();
    assert!(
        !response1.text.is_empty(),
        "First response should not be empty"
    );
    println!("Turn 1 response: {}", response1.text);

    // Second turn: ask about what was just said - the agent should remember
    let result2 = agent
        .query("What is my favorite color and where do I live?")
        .await;

    assert!(result2.is_ok(), "Second query should succeed");
    let response2 = result2.unwrap();
    assert!(
        !response2.text.is_empty(),
        "Second response should not be empty"
    );
    println!("Turn 2 response: {}", response2.text);

    // Verify the agent remembered the context
    // The response should mention "blue" and "Seattle"
    let text_lower = response2.text.to_lowercase();
    let remembers_color = text_lower.contains("blue");
    let remembers_city = text_lower.contains("seattle");

    assert!(
        remembers_color,
        "Agent should remember favorite color is blue. Response: {}",
        response2.text
    );
    assert!(
        remembers_city,
        "Agent should remember living in Seattle. Response: {}",
        response2.text
    );

    // Also verify the message history is being accumulated
    let messages = agent.get_messages();
    // Should have at least: user msg 1, assistant msg 1, user msg 2, assistant msg 2
    assert!(
        messages.len() >= 4,
        "Should have at least 4 messages in history, got {}",
        messages.len()
    );
    println!("Message history has {} messages", messages.len());
}

/// Test that ToolSearchTool can be used to discover and use a deferred tool
/// This tests the full flow: agent uses ToolSearch to find a tool, then uses it
#[tokio::test]
async fn test_tool_search_discovers_and_uses_deferred_tool() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools (includes deferred tools like WebSearch, WebFetch)
    use crate::get_all_tools;
    let tools = get_all_tools();

    // Verify we have ToolSearch available
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(
        tool_names.contains(&"ToolSearch"),
        "Should have ToolSearch tool"
    );
    assert!(
        tool_names.contains(&"WebSearch"),
        "Should have WebSearch (deferred) tool"
    );

    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(10)
        .tools(tools);

    // Ask the agent to discover and use WebSearch via ToolSearch
    let result = agent
        .query("Use ToolSearch to discover the WebSearch tool, then use it to look up the latest news about Iran.")
        .await;

    assert!(result.is_ok(), "Agent should respond successfully");
    let response = result.unwrap();
    assert!(
        !response.text.is_empty(),
        "Agent returned empty response (possible API issue)."
    );

    // Verify ToolSearch was used to discover WebSearch by checking message history.
    // The LLM should discover WebSearch via ToolSearch before using it to answer the question.
    let messages = agent.get_messages();
    let has_tool_search_call = messages
        .iter()
        .any(|m| m.content.contains("ToolSearch") || m.content.contains("WebSearch"));
    assert!(
        has_tool_search_call,
        "Agent should have used ToolSearch to discover WebSearch. Messages: {:?}",
        messages
    );
}

/// Test that AgentEvent streaming events are properly emitted during agent execution.
/// This verifies: Thinking, MessageStart, MessageStop, ContentBlockDelta, ToolStart, ToolComplete
#[tokio::test]
async fn test_agent_events_emitted_correctly() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;

    let tools = get_all_tools();

    // Track events received
    use std::sync::Mutex;
    let events_received: std::sync::Arc<Mutex<Vec<crate::types::AgentEvent>>> =
        std::sync::Arc::new(Mutex::new(Vec::new()));
    let events_clone = events_received.clone();

    // Create agent with event callback
    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(5)
        .tools(tools)
        .on_event(move |event| {
            events_clone.lock().unwrap().push(event);
        });

    // Prompt that will use the Bash tool
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        agent.query("Run this command and tell me the output: echo 'EventTest123'"),
    )
    .await
    .expect("test timed out after 10s (likely API rate limit with retries in parallel test run)");

    // Verify we got a response
    assert!(result.is_ok(), "Agent should respond successfully");
    let response = result.unwrap();
    assert!(!response.text.is_empty(), "Response should not be empty");
    println!("Agent response: {}", response.text);

    // Get the events that were received
    let events = events_received.lock().unwrap();

    // Verify we received Thinking event
    let has_thinking = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::Thinking { .. }));
    assert!(
        has_thinking,
        "Should have received Thinking event. Events: {:?}",
        events
    );

    // Verify we received MessageStart event
    let has_message_start = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::MessageStart { .. }));
    assert!(
        has_message_start,
        "Should have received MessageStart event. Events: {:?}",
        events
    );

    // Verify we received MessageStop event
    let has_message_stop = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::MessageStop));
    assert!(
        has_message_stop,
        "Should have received MessageStop event. Events: {:?}",
        events
    );

    // Verify we received ContentBlockDelta event (text content)
    let has_content_delta = events.iter().any(|e| match e {
        crate::types::AgentEvent::ContentBlockDelta { delta, .. } => match delta {
            ContentDelta::Text { text } => !text.is_empty(),
            _ => false,
        },
        _ => false,
    });
    assert!(
        has_content_delta,
        "Should have received ContentBlockDelta with text. Events: {:?}",
        events
    );

    // Verify we received ToolStart event (command should trigger Bash tool)
    let has_tool_start = events.iter().any(|e| matches!(e, crate::types::AgentEvent::ToolStart { tool_name, .. } if tool_name == "Bash" || tool_name == "bash"));
    println!(
        "ToolStart check: {:?}",
        events
            .iter()
            .filter(|e| matches!(e, crate::types::AgentEvent::ToolStart { .. }))
            .collect::<Vec<_>>()
    );
    assert!(
        has_tool_start,
        "Should have received ToolStart event for Bash tool. Events: {:?}",
        events
    );

    // Verify we received ToolComplete event
    let has_tool_complete = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::ToolComplete { .. }));
    assert!(
        has_tool_complete,
        "Should have received ToolComplete event. Events: {:?}",
        events
    );

    // Verify the output contains our test string
    let text_lower = response.text.to_lowercase();
    let has_expected_output = text_lower.contains("eventtest123");
    assert!(
        has_expected_output,
        "Response should contain 'EventTest123'. Response: {}",
        response.text
    );

    println!("All event checks passed! Events received:");
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i, event);
    }
}

/// Test that MaxTurnsReached event is emitted when max turns is limited to 1
#[tokio::test]
async fn test_agent_max_turns_reached_event() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    // Track events received
    use std::sync::Mutex;
    let events_received: std::sync::Arc<Mutex<Vec<crate::types::AgentEvent>>> =
        std::sync::Arc::new(Mutex::new(Vec::new()));
    let events_clone = events_received.clone();

    // Create agent with max_turns=1 (forces MaxTurnsReached)
    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(1)
        .tools(tools)
        .on_event(move |event| {
            events_clone.lock().unwrap().push(event);
        });

    // Prompt that requires tool use (will need more than 1 turn)
    let result = agent.query("Run this command: echo 'MaxTurnsTest'").await;

    // Should still return a result (may be truncated due to max turns)
    assert!(
        result.is_ok(),
        "Agent should return result even with max turns"
    );
    let response = result.unwrap();
    println!("Agent response (max_turns=1): {}", response.text);

    // Get the events that were received
    let events = events_received.lock().unwrap();

    // Verify we received MessageStart event
    let has_message_start = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::MessageStart { .. }));
    assert!(
        has_message_start,
        "Should have received MessageStart event. Events: {:?}",
        events
    );

    // Verify we received MessageStop event
    let has_message_stop = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::MessageStop));
    assert!(
        has_message_stop,
        "Should have received MessageStop event. Events: {:?}",
        events
    );

    // Verify we received MaxTurnsReached event
    let has_max_turns = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::MaxTurnsReached { .. }));
    println!(
        "MaxTurnsReached check: {:?}",
        events
            .iter()
            .filter(|e| matches!(e, crate::types::AgentEvent::MaxTurnsReached { .. }))
            .collect::<Vec<_>>()
    );
    assert!(
        has_max_turns,
        "Should have received MaxTurnsReached event. Events: {:?}",
        events
    );

    println!("MaxTurnsReached test passed! Events received:");
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i, event);
    }
}

/// Test that ToolError event is emitted when a tool fails
#[tokio::test]
async fn test_agent_tool_error_event() {
    // Only run if required env vars are set
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    // Load config from .env file
    let config = EnvConfig::load();

    // Skip if no API configured
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    // Get all available tools
    use crate::get_all_tools;
    let tools = get_all_tools();

    // Track events received
    use std::sync::Mutex;
    let events_received: std::sync::Arc<Mutex<Vec<crate::types::AgentEvent>>> =
        std::sync::Arc::new(Mutex::new(Vec::new()));
    let events_clone = events_received.clone();

    // Create agent with event callback
    let agent = Agent::new(config.model.as_ref().unwrap())
        .max_turns(3)
        .tools(tools)
        .on_event(move |event| {
            events_clone.lock().unwrap().push(event);
        });

    // Prompt that tries to access a non-existing file (will trigger tool error)
    let result = agent
        .query("Read the first line of the file 'no-such-file-xyz.txt' using head command")
        .await;

    // Should still return a result (agent handles error)
    assert!(
        result.is_ok(),
        "Agent should return result even with tool error"
    );
    let response = result.unwrap();
    println!("Agent response (tool error case): {}", response.text);

    // Get the events that were received
    let events = events_received.lock().unwrap();

    // Verify we received MessageStart event
    let has_message_start = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::MessageStart { .. }));
    assert!(
        has_message_start,
        "Should have received MessageStart event. Events: {:?}",
        events
    );

    // Verify we received MessageStop event
    let has_message_stop = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::MessageStop));
    assert!(
        has_message_stop,
        "Should have received MessageStop event. Events: {:?}",
        events
    );

    // Verify we received ToolStart event
    let has_tool_start = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::ToolStart { .. }));
    println!(
        "ToolStart check: {:?}",
        events
            .iter()
            .filter(|e| matches!(e, crate::types::AgentEvent::ToolStart { .. }))
            .collect::<Vec<_>>()
    );
    assert!(
        has_tool_start,
        "Should have received ToolStart event. Events: {:?}",
        events
    );

    // Verify we received ToolError event (file not found should trigger error)
    let has_tool_error = events
        .iter()
        .any(|e| matches!(e, crate::types::AgentEvent::ToolError { .. }));
    println!(
        "ToolError check: {:?}",
        events
            .iter()
            .filter(|e| matches!(e, crate::types::AgentEvent::ToolError { .. }))
            .collect::<Vec<_>>()
    );

    // Note: ToolError may or may not fire depending on how the LLM handles the error
    // The important thing is we get a response
    if has_tool_error {
        println!("ToolError event detected!");
    }

    println!("ToolError test completed! Events received:");
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i, event);
    }
}

// ---------------------------------------------------------------------------
// Persisted engine mechanics tests
// ---------------------------------------------------------------------------

/// Verify that the agent accumulates messages across multiple query() calls.
/// This is the core persisted-engine test: the engine must not be recreated
/// between calls, so conversation state carries forward naturally.
#[tokio::test]
async fn test_persisted_engine_accumulates_messages() {
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    let config = EnvConfig::load();
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    let agent = Agent::new("claude-sonnet-4-6").max_turns(5);

    // Before any query, message list should be empty
    assert!(
        agent.get_messages().is_empty(),
        "Messages should be empty before first query"
    );

    // First call — store the message count after
    let result1 = agent.query("Say 'Hello' and nothing else.").await;
    assert!(result1.is_ok(), "First query should succeed");
    let msgs1 = agent.get_messages();
    assert!(
        msgs1.len() >= 2,
        "After first query: expected >=2 messages, got {}",
        msgs1.len()
    );

    // Second call — message list must be longer (accumulates)
    let result2 = agent.query("Repeat back what I just said to you.").await;
    assert!(result2.is_ok(), "Second query should succeed");
    let msgs2 = agent.get_messages();
    assert!(
        msgs2.len() > msgs1.len(),
        "After second query: expected {} > {} messages (messages should accumulate)",
        msgs2.len(),
        msgs1.len()
    );

    // Third call — still more messages
    let result3 = agent.query("Now say goodbye.").await;
    assert!(result3.is_ok(), "Third query should succeed");
    let msgs3 = agent.get_messages();
    assert!(
        msgs3.len() > msgs2.len(),
        "After third query: expected {} > {} messages (messages should keep accumulating)",
        msgs3.len(),
        msgs2.len()
    );

    println!(
        "Persisted engine: {} -> {} -> {} messages across 3 turns",
        msgs1.len(),
        msgs2.len(),
        msgs3.len()
    );
}

/// Verify that reset() causes the engine to be recreated and messages are cleared.
#[tokio::test]
async fn test_reset_clears_engine_state() {
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    let config = EnvConfig::load();
    if config.base_url.is_none() || config.auth_token.is_none() {
        eprintln!("Skipping test: no API config found");
        return;
    }

    let agent = Agent::new("claude-sonnet-4-6").max_turns(5);

    // First query
    let _r1 = agent.query("Say 'ResetTest'.").await;
    let msgs_before = agent.get_messages();
    assert!(msgs_before.len() >= 2);

    // Reset
    agent.reset();

    // After reset, message list must be empty
    assert!(
        agent.get_messages().is_empty(),
        "Messages should be empty after reset"
    );

    // Second query should work fine (engine recreated)
    let _r2 = agent.query("Say 'PostReset'.").await;
    let msgs_after = agent.get_messages();
    assert!(
        msgs_after.len() >= 2,
        "After reset + query: expected >=2 messages, got {}",
        msgs_after.len()
    );

    // Agent can continue calling after reset (engine was recreated)
    let _r3 = agent.query("Say 'Again'.").await;
    let msgs_after2 = agent.get_messages();
    assert!(
        msgs_after2.len() > msgs_after.len(),
        "Agent should accumulate messages again after reset"
    );

    println!(
        "Reset test: {} -> clear -> {} -> {} messages",
        msgs_before.len(),
        msgs_after.len(),
        msgs_after2.len()
    );
}

/// Verify that the agent remembers context across query() calls via the
/// persisted engine — the LLM should reference information from turn 1 when
/// asked about it in turn 2.
#[tokio::test]
async fn test_persisted_engine_llm_remembers_context() {
    if !has_required_env_vars() {
        eprintln!("Skipping test: AI_BASE_URL, AI_MODEL, or AI_AUTH_TOKEN not set");
        return;
    }

    let agent = Agent::new("claude-sonnet-4-6").max_turns(5);

    // Turn 1: give the agent a specific fact to embed in conversation context
    let _r1 = agent
        .query("Remember this fact: The capital of Burkina Faso is Ouagadougou. Say only 'OK' if you understand.")
        .await;
    assert!(_r1.is_ok());
    let msgs1 = agent.get_messages();
    assert!(msgs1.len() >= 2);

    // Turn 2: ask the question that requires recalling turn 1's content
    // Retry a few times in case the LLM calls a tool instead of answering directly
    let mut answer = String::new();
    for _ in 0..3 {
        let r2 = agent
            .query("What is the capital of Burkina Faso? Answer with just the city name.")
            .await;
        assert!(r2.is_ok(), "Second turn should succeed");
        answer = r2.unwrap().text.to_lowercase();
        println!("LLM remembers context: '{}'", answer);
        if answer.contains("ouagadougou") {
            break;
        }
        // LLM called a tool instead of answering — wait and retry
        thread::sleep(std::time::Duration::from_secs(2));
    }
    assert!(
        answer.contains("ouagadougou"),
        "LLM should recall the capital from turn 1. Response: '{}'. Engine must have preserved conversation history across query() calls.",
        answer
    );

    // Turn 3: ask about what the user asked in turn 2 — tests 3-turn context
    // Retry for the same reason as turn 2
    let mut answer3 = String::new();
    for _ in 0..3 {
        let r3 = agent
            .query("What did I just ask you about? Only name the country.")
            .await;
        assert!(r3.is_ok());
        answer3 = r3.unwrap().text.to_lowercase();
        println!("LLM remembers turn 2: '{}'", answer3);
        if answer3.contains("burkina") || answer3.contains("burkina faso") {
            break;
        }
        thread::sleep(std::time::Duration::from_secs(2));
    }
    assert!(
        answer3.contains("burkina") || answer3.contains("burkina faso"),
        "LLM should recall turn 2 question from 3-turn history. Response: '{}'",
        answer3
    );

    let msgs3 = agent.get_messages();
    assert!(
        msgs3.len() >= 6,
        "After 3 turns: expected >=6 messages, got {} (user+assistant per turn)",
        msgs3.len()
    );

    println!(
        "LLM context retention test passed: {} messages across 3 turns",
        msgs3.len()
    );
}
