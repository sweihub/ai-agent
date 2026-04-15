# AI Agent SDK (Claude Code SDK in Rust)

[![Crates.io](https://img.shields.io/crates/v/ai-agent)](https://crates.io/crates/ai-agent)
[![Rust](https://img.shields.io/badge/rust-%3E%3D1.70-blue)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

[English](README.md) | [中文](READCN.md)

Idiomatic Rust SDK — 1:1 translation of Claude Code. Runs the full agent loop **in-process** with 25+ built-in tools. Deploy anywhere: cloud, serverless, Docker, CI/CD.

AI Coding CLI: [ai-code](https://github.com/sweihub/ai-code)

## Quick Start

```bash
cargo add ai-agent
export AI_AUTH_TOKEN=your-api-key
export AI_MODEL=MiniMaxAI/MiniMax-M2.5
# Optional: AI_BASE_URL=https://api.minimax.chat/v1
```

```rust
use ai_agent::Agent;
let mut agent = Agent::new("MiniMaxAI/MiniMax-M2.5", 10);
agent.prompt("List 10 files").await?;
```

See [Usage Examples](#usage-examples) for more.

## Core Features

| Feature | Description |
|---------|-------------|
| **Agent** | Create agents with custom models, tools, and prompts |
| **Subagent** | Spawn subagents for parallel or specialized tasks |
| **Session** | Persist, resume, fork conversations on disk |
| **Context Compact** | Automatic conversation summarization when approaching context limits |
| **Skills** | Load external skills or use 15+ bundled skills |
| **Hooks** | 20+ lifecycle events (PreToolUse, PostToolUse, SessionStart, etc.) |
| **Tools** | 25+ built-in tools (Bash, Read, Write, Edit, Glob, Grep, WebFetch, WebSearch, Agent, Tasks, Teams, Worktree, Cron, etc.) |
| **Memory** | File-based persistent context via MEMORY.md |
| **Permissions** | Tool access control with allow/deny rules |
| **Plugins** | Load plugins with commands, skills, MCP servers |
| **MCP** | Connect to Model Context Protocol servers |
| **Cost Tracking** | Real-time token usage and cost estimation |

## Usage Examples

> The agent automatically uses 25+ built-in tools (Bash, Read, Write, Edit, Glob, Grep, WebFetch, etc.) to accomplish tasks.

### Multi-turn Conversation
```rust
let mut agent = Agent::new("MiniMaxAI/MiniMax-M2.5", 5);
agent.prompt("Create /tmp/hello.txt with 'Hello'").await?;
agent.prompt("Read that file back").await?;
println!("Messages: {}", agent.get_messages().len());
```

### Custom Tools
```rust
let calculator = ai_agent::Tool {
    name: "Calculator".into(),
    description: "Evaluate math expressions".into(),
    input_schema: ToolInputSchema::Json(serde_json::json!({
        "type": "object",
        "properties": {"expression": {"type": "string"}},
        "required": ["expression"]
    })),
    executor: Box::new(|input, _ctx| async move {
        Ok(ToolResult { /* ... */ })
    }),
};
```

### MCP Servers
```rust
let config = McpServerConfig::Stdio(McpStdioConfig {
    command: "npx".into(),
    args: Some(vec!["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]),
    ..Default::default()
});
```

### Hooks
```rust
registry.register("PreToolUse", HookDefinition {
    command: Some("echo pre-tool".into()),
    timeout: Some(5000),
    matcher: Some("Read.*".into()),
});
```

## Configuration

### Agent Options
| Option | Default | Description |
|--------|---------|-------------|
| `model` | MiniMaxAI/MiniMax-M2.5 | LLM model ID |
| `max_turns` | 10 | Max agentic turns |
| `max_tokens` | 16384 | Max response tokens |
| `max_budget_usd` | — | Spending cap |
| `system_prompt` | — | Custom system prompt |
| `cwd` | process.cwd() | Working directory |
| `allowed_tools` | all | Tool allow-list |
| `disallowed_tools` | — | Tool deny-list |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `AI_AUTH_TOKEN` | — | API key (required) |
| `AI_MODEL` | MiniMaxAI/MiniMax-M2.5 | Model name |
| `AI_BASE_URL` | — | Custom API endpoint |
| `AI_CONTEXT_WINDOW` | 200000 | Context window size |
| `AI_DISABLE_AUTO_MEMORY` | false | Disable auto memory |
| `AI_MEMORY_PATH_OVERRIDE` | ~/.ai | Memory directory |
| `AI_AUTO_COMPACT_WINDOW` | model-based | Compact trigger window |
| `AI_AUTOCOMPACT_PCT_OVERRIDE` | — | Threshold % (0-100) |
| `AI_DISABLE_COMPACT` | false | Disable compaction |
| `AI_CODE_DISABLE_BACKGROUND_TASKS` | false | Disable background tasks |

## API Compatibility

SDK uses OpenAI format, compatible with:

- [MiniMax](https://platform.minimax.chat)
- [Anthropic](https://www.anthropic.com) (via compatible endpoint)
- [OpenAI](https://openai.com) (compatible mode)
- Any provider with `/v1/chat/completions` endpoint

## Architecture

```
┌─────────────────────────────────────┐
│         Your Application             │
│   use ai_agent::Agent            │
└──────────────┬──────────────────────┘
               │
    ┌──────────▼──────────┐
    │       Agent         │  Session, tools, MCP
    │    prompt()         │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │    QueryEngine      │  Agent loop: API → tools → repeat
    └──────────┬──────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼───┐  ┌───▼───┐  ┌──▼────┐
│  LLM  │  │ 25+   │  │  MCP  │
│  API  │  │Tools  │  │Server │
└───────┘  └───────┘  └───────┘
```

## Examples

```bash
cargo run --example 01_simple_query
cargo run --example 18_plugin
cargo run --example 19_hooks
```

## License

MIT
