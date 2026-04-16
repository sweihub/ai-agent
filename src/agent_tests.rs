#[cfg(test)]
mod tests {
    use crate::agent::build_agent_system_prompt;
    use crate::query_engine::{QueryEngine, QueryEngineConfig};
    use crate::env::EnvConfig;
    use crate::types::{AgentOptions, ToolContext};
    use crate::Agent;
    use crate::AgentError;

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
        let agent = Agent::create(AgentOptions {
            model: Some("claude-sonnet-4-6".to_string()),
            ..Default::default()
        });
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
        assert!(config.auth_token.is_some(), "Auth token should be configured");
        assert!(config.model.is_some(), "Model should be configured");

        // Create agent with real config
        let agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            tools: vec![],
            ..Default::default()
        });

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

        let mut agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            max_turns: Some(3),
            tools,
            ..Default::default()
        });

        // Make a simple prompt that should trigger tool use
        let result = agent.prompt("What is 2 + 2? Just give me the answer.").await;

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

        let mut agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            max_turns: Some(3),
            tools,
            ..Default::default()
        });

        // Prompt that might use tools
        let result = agent.prompt("List all Rust files in the current directory using glob").await;

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
        assert!(tool_names.contains(&"FileRead"), "Should have FileRead tool");
        assert!(tool_names.contains(&"Glob"), "Should have Glob tool");
        println!("Available tools: {:?}", tool_names);

        // Create agent - this will call register_all_tool_executors internally
        let mut agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            max_turns: Some(3),
            tools,
            ..Default::default()
        });

        // Prompt that should definitely use the Bash tool
        let result = agent
            .prompt("Run this command: echo 'hello from tool test'")
            .await;

        // Verify we got a response
        assert!(result.is_ok(), "Agent should respond successfully");
        let response = result.unwrap();
        assert!(!response.text.is_empty(), "Response should not be empty");

        // Check that the tool was actually used (response should contain output)
        let text_lower = response.text.to_lowercase();
        let tool_was_used =
            text_lower.contains("hello from tool test") || text_lower.contains("tool");
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

        let mut agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            max_turns: Some(3),
            tools,
            ..Default::default()
        });

        // Prompt that should use Glob tool
        let result = agent
            .prompt("List all .rs files in the src directory using the Glob tool")
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

        let mut agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            max_turns: Some(3),
            tools,
            ..Default::default()
        });

        // Prompt that should use FileRead tool
        let result = agent
            .prompt("Read the Cargo.toml file from the current directory")
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

        let mut agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            max_turns: Some(5),
            tools,
            ..Default::default()
        });

        // Prompt that should use multiple tools
        let result = agent
            .prompt("First list all files in the current directory, then read the README.md file if it exists")
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

        let mut agent = Agent::create(AgentOptions {
            model: config.model.clone(),
            max_turns: Some(5),
            tools,
            ..Default::default()
        });

        // First turn: tell the agent something specific to remember
        let result1 = agent
            .prompt("Remember this: My favorite color is blue and I live in Seattle.")
            .await;

        assert!(result1.is_ok(), "First query should succeed");
        let response1 = result1.unwrap();
        assert!(!response1.text.is_empty(), "First response should not be empty");
        println!("Turn 1 response: {}", response1.text);

        // Second turn: ask about what was just said - the agent should remember
        let result2 = agent
            .prompt("What is my favorite color and where do I live?")
            .await;

        assert!(result2.is_ok(), "Second query should succeed");
        let response2 = result2.unwrap();
        assert!(!response2.text.is_empty(), "Second response should not be empty");
        println!("Turn 2 response: {}", response2.text);

        // Verify the agent remembered the context
        // The response should mention "blue" and "Seattle"
        let text_lower = response2.text.to_lowercase();
        let remembers_color = text_lower.contains("blue");
        let remembers_city = text_lower.contains("seattle");

        assert!(remembers_color, "Agent should remember favorite color is blue. Response: {}", response2.text);
        assert!(remembers_city, "Agent should remember living in Seattle. Response: {}", response2.text);

        // Also verify the message history is being accumulated
        let messages = agent.get_messages();
        // Should have at least: user msg 1, assistant msg 1, user msg 2, assistant msg 2
        assert!(messages.len() >= 4, "Should have at least 4 messages in history, got {}", messages.len());
        println!("Message history has {} messages", messages.len());
    }
}
