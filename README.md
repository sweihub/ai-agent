# AI Agent SDK (Claude Code SDK in Rust)

[![Crates.io](https://img.shields.io/crates/v/ai-agent)](https://crates.io/crates/ai-agent)
[![Rust](https://img.shields.io/badge/rust-%3E%3D1.70-blue)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

[English](README.md) | [中文](READCN.md)

Idiomatic Rust SDK — 1:1 translation of Claude Code. Runs the full agent loop **in-process** with 50 built-in tools. Deploy anywhere: cloud, serverless, Docker, CI/CD.

AI Coding CLI: [ai-code](https://github.com/sweihub/ai-code)

## Quick Start

```bash
cargo add ai-agent
export AI_AUTH_TOKEN=your-api-key
export AI_MODEL=MiniMaxAI/MiniMax-M2.5
```

```rust
use ai_agent::Agent;

// Simple one-shot query
let answer = Agent::prompt("claude-sonnet-4-6", "List 10 files").await?;

// Full agent with builder pattern
let agent = Agent::new("claude-sonnet-4-6")
    .max_turns(10);
let result = agent.query("List 10 files").await?;
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
| **Hooks** | 29 lifecycle events with 4 hook types (shell, HTTP, prompt, agent), parallel execution, `if` conditions |
| **Tools** | 50 built-in tools across 15 categories (File Ops, Discovery, Shell, Web, LSP, Multi-agent, Tasks, Planning, User Interaction, Scheduling, Git, Skills, MCP, Remote, Communication) |
| **Memory** | File-based persistent context via MEMORY.md |
| **Permissions** | Tool access control with allow/deny rules |
| **Plugins** | Load plugins with commands, skills, MCP servers |
| **MCP** | Connect to Model Context Protocol servers |
| **Cost Tracking** | Real-time token usage and cost estimation |

## Built-in Tools

The SDK ships with **50 built-in tools** organized into 15 categories. All tools are available out of the box with full parameter validation and type-safe schemas.

### File Operations (4)
| # | Tool | Description |
|---|------|-------------|
| 1 | `Read` | Read files from filesystem — supports text files, images (PNG, JPG, GIF, WebP), PDFs, and Jupyter notebooks. Use offset and limit for large files |
| 2 | `Write` | Write content to files with exact path control |
| 3 | `FileEdit` | Edit files by performing exact string replacements (single or all occurrences) |
| 4 | `NotebookEdit` | Edit Jupyter notebook (.ipynb) cells — replace, insert, or delete cell content |

### File Discovery (2)
| # | Tool | Description |
|---|------|-------------|
| 5 | `Glob` | Find files by glob pattern (e.g. `**/*.ts`) for file discovery |
| 6 | `Grep` | Search file contents using regex patterns — uses ripgrep (rg) if available, falls back to grep |

### Shell Execution (3)
| # | Tool | Description |
|---|------|-------------|
| 7 | `Bash` | Execute shell commands with sandboxing, timeouts, and destructive command safety checks |
| 8 | `PowerShell` | Execute PowerShell commands — Windows-only tool for cmdlets and native executable execution |
| 9 | `Sleep` | Wait for a specified duration — user-interruptible, doesn't hold a shell process |

### Web (3)
| # | Tool | Description |
|---|------|-------------|
| 10 | `WebFetch` | Fetch content from a URL and return as text — supports HTML (strips tags), JSON APIs, and plain text |
| 11 | `WebSearch` | Search the web for information — returns titles, URLs, and snippets |
| 12 | `WebBrowser` | Control a web browser for automation (navigate, screenshot, click, fill, evaluate JS) |

### Code Intelligence (1)
| # | Tool | Description |
|---|------|-------------|
| 13 | `LSP` | Language Server Protocol operations — go-to-definition, find references, hover docs, document/workspace symbols, call hierarchy, implementations |

### Multi-agent Orchestration (4)
| # | Tool | Description |
|---|------|-------------|
| 14 | `Agent` | Launch a subagent to handle complex, multi-step tasks autonomously (Explore, Plan, code-reviewer, general-purpose types) |
| 15 | `TeamCreate` | Create a team of agents that can work in parallel |
| 16 | `TeamDelete` | Delete a previously created agent team |
| 17 | `SendMessage` | Send a message to another agent within a team |

### Task Management (6)
| # | Tool | Description |
|---|------|-------------|
| 18 | `TaskCreate` | Create a new structured task with subject, description, and active form |
| 19 | `TaskList` | List all tasks with statuses, owners, and dependencies |
| 20 | `TaskUpdate` | Update task status, details, or dependencies (pending → in_progress → completed) |
| 21 | `TaskGet` | Get full details of a specific task by ID |
| 22 | `TaskStop` | Stop a running background task by ID (also accepts shell_id for KillShell compatibility) |
| 23 | `TaskOutput` | Retrieve output from a running or completed background task with configurable timeout |

### Planning Mode (2)
| # | Tool | Description |
|---|------|-------------|
| 24 | `EnterPlanMode` | Enter structured planning mode to explore codebase and design implementation |
| 25 | `ExitPlanMode` | Exit planning mode and present the plan for user approval |

### User Interaction (2)
| # | Tool | Description |
|---|------|-------------|
| 26 | `AskUserQuestion` | Ask the user a question with multiple-choice options, previews, and multi-select support |
| 27 | `SendUserMessage` | Send a message to the user that they will actually read — brief, digest-style output |

### Scheduling (3)
| # | Tool | Description |
|---|------|-------------|
| 28 | `CronCreate` | Schedule a task using standard 5-field cron expressions — supports recurring (7-day max) and one-shot modes |
| 29 | `CronDelete` | Cancel a previously scheduled cron task |
| 30 | `CronList` | List all scheduled cron tasks (both durable and session-only) |

### Git Worktrees (2)
| # | Tool | Description |
|---|------|-------------|
| 31 | `EnterWorktree` | Create and enter an isolated git worktree for feature development |
| 32 | `ExitWorktree` | Exit a worktree session — option to keep or remove the worktree directory |

### Skills & Configuration (4)
| # | Tool | Description |
|---|------|-------------|
| 33 | `Skill` | Invoke a skill by name — pre-built workflows like brainstorming, TDD, debugging, security-review |
| 34 | `Config` | Read or update dynamic harness configuration (permissions, hooks, env vars) |
| 35 | `ToolSearch` | Search for available tools by name or description |
| 36 | `TodoWrite` | Update the session todo list — track progress and organize multi-step work |

### MCP (Model Context Protocol) (4)
| # | Tool | Description |
|---|------|-------------|
| 37 | `MCPTool` | Execute a tool on an MCP server — dynamically registered with `mcp__serverName_toolName` dispatch |
| 38 | `McpAuth` | Authenticate an MCP server requiring OAuth — returns an authorization URL for the user |
| 39 | `ListMcpResourcesTool` | List available resources from configured MCP servers |
| 40 | `ReadMcpResourceTool` | Read a specific resource from an MCP server by URI |

### Remote / Cloud (1)
| # | Tool | Description |
|---|------|-------------|
| 41 | `RemoteTrigger` | Manage scheduled remote Claude Code agents (triggers) via the claude.ai CCR API — list, create, update, run |

### Communication & Data (3)
| # | Tool | Description |
|---|------|-------------|
| 42 | `StructuredOutput` | Return structured output in a requested format — called exactly once at end of response |
| 43 | `send_user_file` | Send a file from the user to the agent |
| 44 | `Monitor` | Monitor system resources and performance |

### Internal / Not Implemented (6)
These tools are defined in the schema but not actively registered. They are placeholders for future features.

| # | Tool | Description |
|---|------|-------------|
| 45 | `DiscoverSkills` | On-demand skill discovery (not yet implemented) |
| 46 | `OverflowTest` | Test overflow behavior (internal test tool) |
| 47 | `ReviewArtifact` | Review artifacts (not yet implemented) |
| 48 | `Snip` | Model-callable compaction tool (not yet implemented) |
| 49 | `TerminalCapture` | Terminal screen capture (not yet implemented) |
| 50 | `Workflow` | Manage workflows (not yet implemented) |

## Usage Examples

> The agent automatically uses 50 built-in tools across 15 categories to accomplish tasks.

### Multi-turn Conversation
```rust
let agent = Agent::new("claude-sonnet-4-6")
    .max_turns(10);
agent.query("Create /tmp/hello.txt with 'Hello'").await?;
agent.query("Read that file back").await?;
println!("Messages: {}", agent.get_messages().len());
```

### Custom Tools
```rust
use ai_agent::{Agent, ToolDefinition, ToolInputSchema};

let agent = Agent::new("claude-sonnet-4-6")
    .max_turns(5)
    .tools(vec![
        ToolDefinition {
            name: "calculator".into(),
            description: "Evaluate a math expression. Returns the result".into(),
            input_schema: ToolInputSchema {
                schema_type: "object".into(),
                properties: serde_json::json!({
                    "expression": {
                        "description": "The expression to evaluate",
                        "type": "string"
                    }
                }),
                required: Some(vec!["expression".into()]),
            },
            ..Default::default()
        },
    ]);
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

### Event Callback API (CLI/TUI Integration)

Register an `on_event` callback to receive incremental events during query execution — ideal for real-time chat UIs and TUIs.

```rust
use ai_agent::{Agent, AgentEvent, ContentDelta};

let agent = Agent::new("claude-sonnet-4-6")
    .max_turns(10)
    .on_event(|event| match &event {
        AgentEvent::ContentBlockDelta {
            delta: ContentDelta::Text { text },
            ..
        } => print!("{}", text),
        AgentEvent::Thinking { turn } => {
            eprintln!("[Turn {} thinking...]", turn);
        }
        AgentEvent::Done { result } => {
            println!("\nDone! Turns: {}", result.num_turns);
        }
        _ => {}
    });

let result = agent.query("write hello world").await?;
```

### Pub/Sub Event Subscription

For decoupled architectures, use `subscribe()` to listen to events independently:

```rust
let (mut sub, _guard) = agent.subscribe();
// Run query in background, consume events via sub.next().await
```

### Interrupting Agent Execution

Call `agent.interrupt()` from another task to cancel a running `query()`.
The operation returns `AgentError::UserAborted`.

```rust
use ai_agent::Agent;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

let agent = Arc::new(Mutex::new(Agent::new("claude-sonnet-4-6").max_turns(10)));
let interrupt_agent = Arc::clone(&agent);

// Spawn a task that interrupts after 5 seconds
let interrupt_task = tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(5)).await;
    interrupt_agent.lock().await.interrupt();
});

// Run the query with exclusive access
let result = {
    let mut ag = agent.lock().await;
    ag.query("Process a large codebase").await
};

let _ = tokio::time::timeout(Duration::from_secs(10), interrupt_task).await;
```

See `examples/27_interrupt.rs` for a full runnable example.

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
    │    query()          │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │    QueryEngine      │  Agent loop: API → tools → repeat
    └──────────┬──────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼───┐  ┌───▼────┐  ┌──▼────┐
│  LLM  │  │ 50     │  │  MCP  │
│  API  │  │Tools   │  │Server │
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
