# od-cli-scanner

> **Rust-native CLI agent detection engine.** Detect installed AI coding agents (Claude Code, Codex, Kimi, Hermes, Reasonix, and more) by probing their binaries, versions, auth status, and available models.

[![CI](https://github.com/nexu-io/od-cli-scanner/actions/workflows/ci.yml/badge.svg)](https://github.com/nexu-io/od-cli-scanner/actions)
[![Crates.io](https://img.shields.io/crates/v/od-cli-scanner)](https://crates.io/crates/od-cli-scanner)
[![npm](https://img.shields.io/npm/v/od-cli-scanner)](https://www.npmjs.com/package/od-cli-scanner)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](#quick-start) | [中文](#快速开始)

---

## Features

- **Concurrent scanning** — Probes all agents in parallel, yields results as they resolve
- **Smart PATH resolution** — Searches user toolchain dirs (Homebrew, ~/.local/bin, cargo, nvm, bun)
- **Auth diagnostics** — Detects "installed but not authenticated" states
- **Live model probing** — Real-time model list fetching via CLI commands
- **Capability detection** — `--help` probing to discover supported flags
- **Agent registry** — Query and manage detected agents programmatically
- **Launch support** — Build args and spawn agents for downstream consumption
- **Env overrides** — Per-agent `*_BIN` environment variables for custom paths
- **Multiple output formats** — JSON, table, CSV
- **Cross-platform** — Linux, macOS, Windows
- **Programmatic API** — Use as a Rust library or Node.js package via napi-rs

## Quick Start

### As a CLI tool

```bash
# Install from crates.io
cargo install od-cli-scanner

# Scan all agents
od-scan scan

# Pretty JSON output
od-scan scan --pretty

# Only available agents, table format
od-scan scan --available-only --format table

# Check a specific binary
od-scan check claude -- --version

# Custom config
od-scan scan --config ./my-agents.json
```

### As a Rust library

```rust
use od_cli_scanner::{detect_agents, AgentRegistry, AgentDef, BuildArgsOptions};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // 1. Detect agents
    let registry = AgentRegistry::new();
    let result = detect_agents(registry.list(), &HashMap::new()).await;
    
    // 2. Find first available agent
    let agent = result.agents.iter().find(|a| a.available).unwrap();
    
    // 3. Get agent definition
    let def = registry.get(&agent.id).unwrap();
    
    // 4. Build launch arguments
    let options = BuildArgsOptions {
        prompt: "Hello world".to_string(),
        model: Some("claude-sonnet-4".to_string()),
        ..Default::default()
    };
    let args = def.build_args(&options);
    println!("Launch args: {:?}", args);
}
```

### As a Node.js package

```bash
npm install od-cli-scanner
```

```javascript
const { detectAgents } = require('od-cli-scanner');

const result = await detectAgents([
  { id: 'claude', name: 'Claude Code', bin: 'claude' }
]);
console.log(JSON.parse(result));
```

## Architecture

| Layer | Responsibility |
|-------|---------------|
| `core::types` | Agent definitions, detection results, diagnostics |
| `core::executables` | PATH resolution, env overrides, toolchain discovery |
| `core::probe` | Async process spawning, version/auth/model/capability probing |
| `core::detector` | Concurrent orchestration, fault isolation |
| `core::registry` | Agent registry with `get()` / `list()` APIs |
| `core::build_args` | Per-agent argument construction for launch |
| `core::spawn` | Process spawning with env/cwd support |
| `cli` | clap-based command-line interface |
| `bindings` | napi-rs Node.js bindings |

## Built-in Agents

| Agent | Binary | Fallback Bins | Stream Format |
|-------|--------|--------------|---------------|
| Claude Code | `claude` | `openclaude` | `claude-stream-json` |
| Codex CLI | `codex` | — | `json-event-stream` |
| Kimi CLI | `kimi` | — | `json-event-stream` |
| Hermes | `hermes` | — | `acp-json-rpc` |
| Reasonix | `reasonix` | `dsnix` | `acp-json-rpc` |

## Configuration

Custom agent definitions via JSON:

```json
[
  {
    "id": "my-agent",
    "name": "My Custom Agent",
    "bin": "my-agent",
    "version_args": ["--version"],
    "fallback_models": [
      { "id": "default", "label": "Default" }
    ],
    "stream_format": "json-event-stream"
  }
]
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `CLAUDE_BIN` | Override Claude Code binary path |
| `CODEX_BIN` | Override Codex CLI binary path |
| `KIMI_BIN` | Override Kimi CLI binary path |
| `HERMES_BIN` | Override Hermes binary path |
| `REASONIX_BIN` | Override Reasonix binary path |

## Development

```bash
# Clone
git clone https://github.com/nexu-io/od-cli-scanner.git
cd od-cli-scanner

# Build
cargo build --release

# Test
cargo test

# Run locally
cargo run -- scan --pretty
```

## Roadmap

- [x] Live model probing (real-time model list fetching)
- [x] Capability flag detection (`--help` probing)
- [x] Agent registry with programmatic APIs
- [x] Agent launch support (build args + spawn)
- [ ] Streaming detection results (SSE/WebSocket)
- [ ] More built-in agents (Cursor, Gemini, Qwen, etc.)
- [ ] WASM target for browser usage

## Acknowledgments

This project was inspired by and built upon the agent detection system from [**Open Design**](https://github.com/nexu-io/open-design) by [nexu-io](https://github.com/nexu-io). The built-in agent definitions, version probing strategies, and stream format mappings are derived from their comprehensive TypeScript runtime registry.

## License

MIT — see [LICENSE](LICENSE)

---

## 致谢

本项目受到 [**Open Design**](https://github.com/nexu-io/open-design)（由 [nexu-io](https://github.com/nexu-io) 开发）的启发并基于其构建。内置的 Agent 定义、版本探测策略和流格式映射均源自其全面的 TypeScript 运行时注册表。

---

## 快速开始

### 作为 CLI 工具

```bash
# 从 crates.io 安装
cargo install od-cli-scanner

# 扫描所有 Agent
od-scan scan

# 漂亮 JSON 输出
od-scan scan --pretty

# 仅显示可用 Agent，表格格式
od-scan scan --available-only --format table
```

### 作为 Rust 库

```rust
use od_cli_scanner::{detect_agents, AgentRegistry, BuildArgsOptions};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // 1. 检测 Agent
    let registry = AgentRegistry::new();
    let result = detect_agents(registry.list(), &HashMap::new()).await;
    
    // 2. 查找第一个可用 Agent
    let agent = result.agents.iter().find(|a| a.available).unwrap();
    
    // 3. 获取 Agent 定义
    let def = registry.get(&agent.id).unwrap();
    
    // 4. 构建启动参数
    let options = BuildArgsOptions {
        prompt: "Hello world".to_string(),
        model: Some("claude-sonnet-4".to_string()),
        ..Default::default()
    };
    let args = def.build_args(&options);
    println!("启动参数: {:?}", args);
}
```

### 作为 Node.js 包

```bash
npm install od-cli-scanner
```

```javascript
const { detectAgents } = require('od-cli-scanner');

const result = await detectAgents([
  { id: 'claude', name: 'Claude Code', bin: 'claude' }
]);
console.log(JSON.parse(result));
```
