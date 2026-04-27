# AI Agent SDK (Rust 版 Claude Code SDK)

[![Crates.io](https://img.shields.io/crates/v/ai-agent)](https://crates.io/crates/ai-agent)
[![Rust](https://img.shields.io/badge/rust-%3E%3D1.70-blue)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

[English](README.md) | [中文](READCN.md)

idiomatic Rust SDK — Claude Code 的 1:1 翻译。**进程内**运行完整 agent 循环，内置 50 个工具。可部署到任意环境：云、无服务器、Docker、CI/CD。

AI Coding CLI: [ai-code](https://github.com/sweihub/ai-code)

## 快速开始

```bash
cargo add ai-agent
export AI_AUTH_TOKEN=your-api-key
export AI_MODEL=MiniMaxAI/MiniMax-M2.5
```

```rust
use ai_agent::Agent;

// 简单的一次性查询
let answer = Agent::prompt("claude-sonnet-4-6", "列出10个文件").await?;

// 完整 Agent，使用 builder 模式
let agent = Agent::new("claude-sonnet-4-6")
    .max_turns(10);
let result = agent.query("列出10个文件").await?;
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
| **Hooks** | 29 个生命周期事件，4 种钩子类型（shell、HTTP、prompt、agent），并行执行，支持 if 条件 |
| **Tools** | 50 内置工具，15 大类别（文件操作、发现、Shell、Web、LSP、多 Agent、任务管理、规划、用户交互、调度、Git、技能、MCP、远程、通信） |
| **Memory** | 通过 MEMORY.md 进行基于文件的持久化上下文 |
| **Permissions** | 工具访问控制，支持允许/拒绝规则 |
| **Plugins** | 加载包含命令、技能、MCP 服务器的插件 |
| **MCP** | 连接到 Model Context Protocol 服务器 |
| **Cost Tracking** | 实时令牌使用量和成本估算 |

## 内置工具

SDK 内置 **50 个工具**，分为 15 大类别。所有工具开箱即用，带有完整的参数验证和类型安全 Schema。

### 文件操作 (4)
| # | 工具 | 描述 |
|---|------|------|
| 1 | `Read` | 读取文件——支持文本、图片（PNG、JPG、GIF、WebP）、PDF、Jupyter 笔记本，大文件支持 offset 和 limit |
| 2 | `Write` | 向文件写入内容，精确控制路径 |
| 3 | `FileEdit` | 在文件中执行精确字符串替换（单次或全部匹配） |
| 4 | `NotebookEdit` | 编辑 Jupyter 笔记本 (.ipynb) 单元格——替换、插入或删除 |

### 文件发现 (2)
| # | 工具 | 描述 |
|---|------|------|
| 5 | `Glob` | 按 glob 模式查找文件（如 `**/*.ts`） |
| 6 | `Grep` | 通过正则表达式搜索文件内容——优先使用 ripgrep (rg)，回退到 grep |

### Shell 执行 (3)
| # | 工具 | 描述 |
|---|------|------|
| 7 | `Bash` | 执行 Shell 命令，内置沙箱、超时控制和破坏性命令安全检查 |
| 8 | `PowerShell` | 执行 PowerShell 命令——Windows 专用，支持 cmdlet 和本地可执行文件 |
| 9 | `Sleep` | 等待指定时长——用户可随时中断，不占用 Shell 进程 |

### Web (3)
| # | 工具 | 描述 |
|---|------|------|
| 10 | `WebFetch` | 从 URL 获取并提取内容——支持 HTML（去除标签）、JSON API、纯文本 |
| 11 | `WebSearch` | 搜索网络获取信息——返回标题、URL 和摘要 |
| 12 | `WebBrowser` | 控制浏览器自动化（导航、截图、点击、填写、执行 JS） |

### 代码智能 (1)
| # | 工具 | 描述 |
|---|------|------|
| 13 | `LSP` | 语言服务器协议操作——跳转定义、查找引用、悬停文档、文档/工作空间符号、调用层次、实现查找 |

### 多 Agent 编排 (4)
| # | 工具 | 描述 |
|---|------|------|
| 14 | `Agent` | 启动子 Agent 自主处理复杂多步任务（Explore、Plan、code-reviewer、general-purpose 等类型） |
| 15 | `TeamCreate` | 创建可并行工作的 Agent 团队 |
| 16 | `TeamDelete` | 删除已创建的 Agent 团队 |
| 17 | `SendMessage` | 向团队内的其他 Agent 发送消息 |

### 任务管理 (6)
| # | 工具 | 描述 |
|---|------|------|
| 18 | `TaskCreate` | 创建新的结构化任务，包含主题、描述和活跃形式 |
| 19 | `TaskList` | 列出所有任务的状态、负责人和依赖关系 |
| 20 | `TaskUpdate` | 更新任务状态、详情或依赖关系（pending → in_progress → completed） |
| 21 | `TaskGet` | 获取指定任务的完整详情 |
| 22 | `TaskStop` | 按 ID 停止运行中的后台任务（兼容 KillShell 的 shell_id 参数） |
| 23 | `TaskOutput` | 获取运行中或已完成后台任务的输出，支持可配置超时 |

### 规划模式 (2)
| # | 工具 | 描述 |
|---|------|------|
| 24 | `EnterPlanMode` | 进入规划模式，探索代码库并设计方案 |
| 25 | `ExitPlanMode` | 退出规划模式，提交方案供用户审批 |

### 用户交互 (2)
| # | 工具 | 描述 |
|---|------|------|
| 26 | `AskUserQuestion` | 向用户发起选择题提问，支持预览和多选 |
| 27 | `SendUserMessage` | 向用户发送实际可见的消息——简报、摘要风格输出 |

### 调度 (3)
| # | 工具 | 描述 |
|---|------|------|
| 28 | `CronCreate` | 使用标准 5 位 cron 表达式定时任务——支持周期性（最长 7 天）和一次性模式 |
| 29 | `CronDelete` | 取消已创建的定时任务 |
| 30 | `CronList` | 列出所有定时任务（持久化和会话级） |

### Git Worktree (2)
| # | 工具 | 描述 |
|---|------|------|
| 31 | `EnterWorktree` | 创建并进入隔离的 git worktree 用于功能开发 |
| 32 | `ExitWorktree` | 退出 worktree 会话——可选择保留或删除 worktree 目录 |

### 技能与配置 (4)
| # | 工具 | 描述 |
|---|------|------|
| 33 | `Skill` | 按名称调用技能——预构建工作流如 brainstorming、TDD、debugging、security-review |
| 34 | `Config` | 读取或更新工作区配置（权限、Hooks、环境变量） |
| 35 | `ToolSearch` | 按名称或描述搜索可用工具 |
| 36 | `TodoWrite` | 更新会话待办列表——跟踪进度和组织多步工作 |

### MCP（Model Context Protocol）(4)
| # | 工具 | 描述 |
|---|------|------|
| 37 | `MCPTool` | 在 MCP 服务器上执行工具——动态注册，`mcp__serverName_toolName` 分发 |
| 38 | `McpAuth` | 认证需要 OAuth 的 MCP 服务器——返回授权 URL |
| 39 | `ListMcpResourcesTool` | 列出已配置 MCP 服务器的可用资源 |
| 40 | `ReadMcpResourceTool` | 通过 URI 从 MCP 服务器读取指定资源 |

### 远程 / 云端 (1)
| # | 工具 | 描述 |
|---|------|------|
| 41 | `RemoteTrigger` | 通过 claude.ai CCR API 管理定时远程 Claude Code Agent（triggers）——列表、创建、更新、运行 |

### 通信与数据 (3)
| # | 工具 | 描述 |
|---|------|------|
| 42 | `StructuredOutput` | 以请求的格式返回结构化输出——响应结束时精确调用一次 |
| 43 | `send_user_file` | 将用户文件发送给 Agent |
| 44 | `Monitor` | 监控系统资源和性能 |

### 内部 / 未实现 (6)
以下工具在 Schema 中已定义但未激活注册，为未来功能预留。

| # | 工具 | 描述 |
|---|------|------|
| 45 | `DiscoverSkills` | 按需技能发现（尚未实现） |
| 46 | `OverflowTest` | 测试溢出行为（内部测试工具） |
| 47 | `ReviewArtifact` | 审查制品（尚未实现） |
| 48 | `Snip` | 模型可调用的压缩工具（尚未实现） |
| 49 | `TerminalCapture` | 终端屏幕捕获（尚未实现） |
| 50 | `Workflow` | 管理工作流（尚未实现） |

## 使用示例

> Agent 自动使用 50 个内置工具，覆盖 15 大类别，来完成任务。

### 多轮对话
```rust
let agent = Agent::new("claude-sonnet-4-6")
    .max_turns(10);
agent.query("创建 /tmp/hello.txt 内容为 'Hello'").await?;
agent.query("读取刚才创建的文件").await?;
println!("消息数: {}", agent.get_messages().len());
```

### 自定义工具
```rust
use ai_agent::{Agent, ToolDefinition, ToolInputSchema};

let agent = Agent::new("claude-sonnet-4-6")
    .max_turns(5)
    .tools(vec![
        ToolDefinition {
            name: "calculator".into(),
            description: "计算数学表达式，返回结果".into(),
            input_schema: ToolInputSchema {
                schema_type: "object".into(),
                properties: serde_json::json!({
                    "expression": {
                        "description": "要计算的表达式",
                        "type": "string"
                    }
                }),
                required: Some(vec!["expression".into()]),
            },
            ..Default::default()
        },
    ]);
```

### MCP 服务器
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

### 事件回调 API（CLI/TUI 集成）

注册 `on_event` 回调以在查询执行期间接收增量事件——非常适合实时聊天界面和终端 UI。

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
            eprintln!("[第 {} 轮 思考中...]", turn);
        }
        AgentEvent::Done { result } => {
            println!("\n完成！轮数: {}", result.num_turns);
        }
        _ => {}
    });

let result = agent.query("写一个 hello world").await?;
```

### 发布/订阅事件监听

对于解耦架构，使用 `subscribe()` 独立监听事件：

```rust
let (mut sub, _guard) = agent.subscribe();
// 在后台运行查询，通过 sub.next().await 消费事件
```

### 中断 Agent 执行

从其他任务调用 `agent.interrupt()` 可以取消正在运行的 `query()`。
操作会返回 `AgentError::UserAborted`。

```rust
use ai_agent::Agent;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

let agent = Arc::new(Mutex::new(Agent::new("claude-sonnet-4-6").max_turns(10)));
let interrupt_agent = Arc::clone(&agent);

// 派生一个在 5 秒后中断的任务
let interrupt_task = tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(5)).await;
    interrupt_agent.lock().await.interrupt();
});

// 以独占方式运行 query
let result = {
    let mut ag = agent.lock().await;
    ag.query("处理大型代码库").await
};

let _ = tokio::time::timeout(Duration::from_secs(10), interrupt_task).await;
```

参见 `examples/27_interrupt.rs` 获取完整可运行示例。

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
    │    query()          │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │    QueryEngine      │  Agent 循环: API → tools → repeat
    └──────────┬──────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼───┐  ┌───▼────┐  ┌──▼────┐
│  LLM  │  │ 50     │  │  MCP  │
│  API  │  │Tools   │  │Server │
└───────┘  └───────┘  └───────┘
```

## 示例

```bash
cargo run --example 01_simple_query
cargo run --example 18_plugin
cargo run --example 19_hooks
```

## API 兼容性

SDK 使用 OpenAI 格式与 LLM 通信，兼容：

- [MiniMax](https://platform.minimax.chat)
- [Anthropic](https://www.anthropic.com) (通过兼容端点)
- [OpenAI](https://openai.com) (兼容模式)
- 任何提供 OpenAI `/v1/chat/completions` 端点的提供商

## 许可证

MIT
