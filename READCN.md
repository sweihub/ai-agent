# AI Agent SDK (Rust 版 Claude Code SDK)

[![Crates.io](https://img.shields.io/crates/v/ai-agent)](https://crates.io/crates/ai-agent)
[![Rust](https://img.shields.io/badge/rust-%3E%3D1.70-blue)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

[English](README.md) | [中文](READCN.md)

 idiomatic Rust SDK — Claude Code 的1:1翻译。**进程内**运行完整 agent 循环，内置 25+ 工具。可部署到任意环境：云、无服务器、Docker、CI/CD。

AI Coding CLI: [ai-code](https://github.com/sweihub/ai-code)

## 快速开始

```bash
cargo add ai-agent
export AI_AUTH_TOKEN=your-api-key
export AI_MODEL=MiniMaxAI/MiniMax-M2.5
# 可选：AI_BASE_URL=https://api.minimax.chat/v1
```

```rust
use ai_agent::Agent;
let mut agent = Agent::new("MiniMaxAI/MiniMax-M2.5", 10);
agent.prompt("列出10个文件").await?;
```

更多示例见 [使用示例](#使用示例)。

## 核心功能

| 功能 | 描述 |
|------|------|
| **Agent** | 使用自定义模型、工具和提示词创建 Agent |
| **Subagent** | 生成子 Agent 用于并行或专业任务 |
| **Session** | 在磁盘上持久化、恢复、分叉对话 |
| **Context Compact** | 接近上下文限制时自动对话摘要 |
| **Skills** | 加载外部技能或使用 15+ 内置技能 |
| **Hooks** | 20+ 生命周期事件 (PreToolUse, PostToolUse, SessionStart 等) |
| **Tools** | 25+ 内置工具 (Bash, Read, Write, Edit, Glob, Grep, WebFetch, WebSearch, Agent, Tasks, Teams, Worktree, Cron 等) |
| **Memory** | 通过 MEMORY.md 进行基于文件的持久化上下文 |
| **Permissions** | 工具访问控制，支持允许/拒绝规则 |
| **Plugins** | 加载包含命令、技能、MCP 服务器的插件 |
| **MCP** | 连接到 Model Context Protocol 服务器 |
| **Cost Tracking** | 实时令牌使用量和成本估算 |

## 使用示例

> Agent 自动使用 25+ 内置工具（Bash、Read、Write、Edit、Glob、Grep、WebFetch 等）来完成任务。

### 多轮对话
```rust
let mut agent = Agent::new("MiniMaxAI/MiniMax-M2.5", 5);
agent.prompt("创建 /tmp/hello.txt 内容为 'Hello'").await?;
agent.prompt("读取刚才创建的文件").await?;
println!("消息数: {}", agent.get_messages().len());
```

### 自定义工具
```rust
let calculator = ai_agent::Tool {
    name: "Calculator".into(),
    description: "计算数学表达式".into(),
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

### MCP 服务器
```rust
let config = McpServerConfig::Stdio(McpStdioConfig {
    command: "npx".into(),
    args: Some(vec!["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]),
    ..Default::default()
});
```

## 配置

### Agent 选项
| 选项 | 默认值 | 描述 |
|------|--------|------|
| `model` | MiniMaxAI/MiniMax-M2.5 | LLM 模型 ID |
| `max_turns` | 10 | 最大 agent 轮次 |
| `max_tokens` | 16384 | 最大响应令牌数 |
| `max_budget_usd` | — | 支出上限 |
| `system_prompt` | — | 自定义系统提示词 |
| `cwd` | process.cwd() | 工作目录 |
| `allowed_tools` | 全部 | 允许的工具列表 |
| `disallowed_tools` | — | 禁用的工具列表 |

### 环境变量

| 变量 | 默认值 | 描述 |
|------|--------|------|
| `AI_AUTH_TOKEN` | — | API 密钥 (必填) |
| `AI_MODEL` | MiniMaxAI/MiniMax-M2.5 | 模型名称 |
| `AI_BASE_URL` | — | 自定义 API 端点 |
| `AI_CONTEXT_WINDOW` | 200000 | 上下文窗口大小 |
| `AI_DISABLE_AUTO_MEMORY` | false | 禁用自动记忆 |
| `AI_MEMORY_PATH_OVERRIDE` | ~/.ai | 记忆目录 |
| `AI_AUTO_COMPACT_WINDOW` | 模型默认值 | 压缩触发窗口 |
| `AI_AUTOCOMPACT_PCT_OVERRIDE` | — | 阈值百分比 (0-100) |
| `AI_DISABLE_COMPACT` | false | 禁用压缩 |
| `AI_CODE_DISABLE_BACKGROUND_TASKS` | false | 禁用后台任务 |

## 架构

```
┌─────────────────────────────────────┐
│         你的应用程序                 │
│   use ai_agent::Agent            │
└──────────────┬──────────────────────┘
               │
    ┌──────────▼──────────┐
    │       Agent         │  会话、工具、MCP
    │    prompt()         │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │    QueryEngine      │  Agent 循环: API → tools → repeat
    └──────────┬──────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼───┐  ┌───▼───┐  ┌──▼────┐
│  LLM  │  │ 25+   │  │  MCP  │
│  API  │  │Tools  │  │Server │
└───────┘  └───────┘  └───────┘
```

## 示例

```bash
cargo run --example 01_simple_query
cargo run --example 06_mcp_server
cargo run --example 09_subagents
```

## API 兼容性

SDK 使用 OpenAI 格式与 LLM 通信，兼容：

- [MiniMax](https://platform.minimax.chat)
- [Anthropic](https://www.anthropic.com) (通过兼容端点)
- [OpenAI](https://openai.com) (兼容模式)
- 任何提供 OpenAI `/v1/chat/completions` 端点的提供商

## 许可证

MIT
