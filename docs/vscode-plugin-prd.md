# od-cli-scanner VS Code Extension — PRD

> **Hero Line**: Detect & launch local AI coding agents inside VS Code — one click, zero friction.
> **Hero Line (中文)**: 在 VS Code 内一键检测并启动本地 AI coding agents，零摩擦，零配置。

| | |
|---|---|
| **Product** | od-cli-scanner VS Code Extension |
| **Version** | v0.1.0 (MVP) |
| **Target** | VS Code ≥ 1.85 |
| **Engine** | od-cli-scanner CLI (Rust, ≥ v0.1.3) |
| **Author** | 老顾 (PM) |
| **Date** | 2026-06-24 |

---

## 1. 用户画像 / User Persona

| 属性 | 描述 |
|------|------|
| **角色** | 全栈开发者、AI-native 工程师 |
| **痛点** | 装了多个 AI agent（Claude Code, Codex, Kimi…），但记不住命令、切来切去麻烦 |
| **目标** | 在 IDE 里一眼看到哪些 agent 可用，点一下就启动 |
| **场景** | 打开项目 → 状态栏显示可用 agents → 选一个 → 自动开终端跑起来 |

---

## 2. 用户旅程 / User Journey

### Journey A: 状态栏一键启动 (P0)
```
打开 VS Code → 状态栏右侧显示 "🤖 3 agents"
    ↓ 点击
下拉菜单列出: Claude Code | Codex | Kimi
    ↓ 选择 "Claude Code"
自动创建终端 → 执行 "claude" → 当前工作区已加载
```

### Journey B: 命令面板启动 (P0)
```
Cmd+Shift+P → "AI Agent: Launch..."
    ↓
快速选择框列出可用 agents
    ↓ 选择 + 可选输入 prompt
终端启动 agent 并传入参数
```

### Journey C: 右键文件/文件夹处理 (P1)
```
Explorer 中右键文件/文件夹
    ↓
"Open with AI Agent" → 子菜单列出 agents
    ↓ 选择
终端启动 agent，自动传入选中路径作为 context
```

---

## 3. 功能列表 / Feature List

### P0 — MVP (v0.1.0)

| # | 功能 | 说明 | 验收标准 |
|---|------|------|----------|
| 1 | **状态栏 Agent 指示器** | 右下角显示可用 agent 数量；点击展开下拉 | 实时反映 `od-scan scan --available-only` 结果 |
| 2 | **一键启动 Agent** | 下拉菜单选 agent → 新建终端 → 执行启动命令 | 终端标题含 agent 名，cwd=workspaceRoot |
| 3 | **命令面板集成** | `AI Agent: Launch...` 命令 | 支持快捷键绑定 |
| 4 | **自动检测** | 激活扩展时/定时调用 od-cli-scanner | 检测耗时 < 3s，不阻塞 UI |
| 5 | **错误兜底** | 无可用 agent 时显示 "Install an AI agent →" | 点击跳转 README |

### P1 — 体验增强 (v0.2.0)

| # | 功能 | 说明 |
|---|------|------|
| 6 | **右键菜单** | Explorer 右键文件/文件夹 → "Open with AI Agent" |
| 7 | **Agent 详情页** | 侧边栏树视图展示所有 agent 状态、版本、认证状态 |
| 8 | **最近使用** | 状态栏优先显示最近启动的 agent |
| 9 | **启动配置** | 支持 `settings.json` 配置默认 agent、启动参数 |

### P2 — 高级 (v0.3.0)

| # | 功能 | 说明 |
|---|------|------|
| 10 | **自动检测间隔** | 可配置定时刷新（默认 60s） |
| 11 | **模型选择** | 启动时可选 agent 支持的模型 |
| 12 | **多工作区** | 每个 workspace folder 独立检测 |

---

## 4. 技术架构 / Technical Architecture

```
┌─────────────────────────────────────────┐
│           VS Code Extension             │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐ │
│  │ Status  │ │ Command │ │ Explorer │ │
│  │  Bar    │ │ Palette │ │  Context │ │
│  └────┬────┘ └────┬────┘ └────┬─────┘ │
│       └────────────┴───────────┘       │
│                   │                     │
│           ┌───────┴───────┐            │
│           │  AgentService │            │
│           │  (singleton)  │            │
│           └───────┬───────┘            │
│                   │                     │
│           ┌───────┴───────┐            │
│           │  ScannerBridge │            │
│           │  (child proc) │            │
│           └───────┬───────┘            │
└───────────────────┼─────────────────────┘
                    │
            ┌───────┴───────┐
            │ od-cli-scanner │
            │  (Rust binary) │
            │  od-scan scan  │
            └───────────────┘
```

### 4.1 核心模块

| 模块 | 职责 | VS Code API |
|------|------|-------------|
| `AgentService` | 缓存检测结果、管理状态、暴露 API | — |
| `ScannerBridge` | 调用 `od-scan` 二进制，解析 JSON | `child_process` |
| `StatusBarController` | 状态栏 UI 更新 | `window.createStatusBarItem` |
| `CommandController` | 命令面板注册 | `commands.registerCommand` |
| `TerminalLauncher` | 创建终端并执行命令 | `window.createTerminal` |
| `ConfigManager` | 读写 `settings.json` | `workspace.getConfiguration` |

### 4.2 配置项 (settings.json)

```jsonc
{
  "odScanner.binaryPath": "/usr/local/bin/od-scan",  // 自定义二进制路径
  "odScanner.defaultAgent": "claude",                   // 默认启动 agent
  "odScanner.autoRefresh": true,                        // 自动刷新检测
  "odScanner.refreshInterval": 60,                      // 刷新间隔（秒）
  "odScanner.showUnavailable": false                    // 状态栏显示不可用 agents
}
```

### 4.3 数据流

```
Extension Activate
    ↓
ScannerBridge.scan() → spawn("od-scan scan --format json")
    ↓
Parse JSON → AgentService.update(agents[])
    ↓
StatusBarController.refresh() → 更新状态栏文本/菜单
    ↓
User Click → AgentService.launch(agentId) → TerminalLauncher.spawn(agent)
```

---

## 5. 用户故事 / User Stories

### US-1: 状态栏快速启动
> **As a** developer with multiple AI agents installed
> **I want** to see available agents in the status bar
> **So that** I can launch one without leaving the editor

**AC**:
- 状态栏显示可用 agent 数量（如 "🤖 3"）
- 点击后下拉菜单显示 agent 名称 + 版本
- 选择后 500ms 内创建终端并启动

### US-2: 命令面板启动
> **As a** keyboard-heavy user
> **I want** to launch an agent via command palette
> **So that** I don't need to reach for the mouse

**AC**:
- `Cmd+Shift+P` → "AI Agent: Launch..." 可用
- 支持模糊匹配（如 "claude" 匹配 "Claude Code"）
- 可选输入 prompt，作为启动参数传入

### US-3: 右键文件处理 (P1)
> **As a** developer reviewing a specific file
> **I want** to open it with an AI agent directly
> **So that** I don't need to manually pass file paths

**AC**:
- Explorer 右键菜单新增 "Open with AI Agent"
- 子菜单列出可用 agents
- 启动命令自动包含选中文件/文件夹路径

### US-4: 无 Agent 引导
> **As a** new user with no agents installed
> **I want** to see a helpful message instead of empty UI
> **So that** I know what to install next

**AC**:
- 状态栏显示 "Install an AI agent →"
- 点击打开浏览器到 od-cli-scanner README
- 列出推荐的 agent 安装命令

---

## 6. 界面规格 / UI Spec

### 6.1 状态栏指示器

| 状态 | 显示文本 | Tooltip |
|------|----------|---------|
| 检测中 | `🤖 ...` | "Scanning for AI agents..." |
| N 个可用 | `🤖 N` | "Click to launch an AI agent" |
| 0 个可用 | `🤖 Install →` | "No AI agents detected. Click to install." |
| 错误 | `🤖 !` | "Scanner error: {message}" |

### 6.2 下拉菜单项

```
🤖 Claude Code    v0.8.0    [已认证]
   Codex CLI      v1.2.0    [已认证]
   Kimi           v0.3.1    [未认证]
   ─────────────────────────
   Refresh Agents
   Settings...
```

### 6.3 命令面板流程

```
> AI Agent: Launch...
  → 选择 Agent: [Claude Code | Codex | Kimi | ...]
  → 输入 Prompt (可选): "Refactor this module"
  → 创建终端: "claude Refactor this module"
```

---

## 7. 里程碑 / Milestones

| 里程碑 | 版本 | 日期 | 交付物 |
|--------|------|------|--------|
| **M1: MVP** | v0.1.0 | W1 | P0 功能：状态栏、命令面板、终端启动 |
| **M2: 体验增强** | v0.2.0 | W2 | P1 功能：右键菜单、Agent 详情页、最近使用 |
| **M3: 高级配置** | v0.3.0 | W3 | P2 功能：自动刷新、模型选择、多工作区 |
| **M4: 生态** | v0.4.0 | W4 | 市场发布、文档、CI/CD |

---

## 8. 依赖与约束 / Dependencies

| 依赖 | 版本 | 说明 |
|------|------|------|
| od-cli-scanner | ≥ v0.1.3 | Rust CLI 检测引擎 |
| VS Code API | ≥ 1.85 | `Terminal` 创建、状态栏、命令面板 |
| Node.js | ≥ 18 | 扩展运行时 |

**约束**:
- 扩展包体积 < 5MB（不含预编译二进制）
- 启动检测耗时 < 3s
- 零配置即可工作（自动发现 `od-scan` 在 PATH）

---

## 9. 附录 / Appendix

### A. 推荐 Agent 安装引导

| Agent | 安装命令 |
|-------|----------|
| Claude Code | `npm install -g @anthropic-ai/claude-code` |
| Codex CLI | `npm install -g @openai/codex` |
| Kimi | `pip install kimi-cli` |
| Hermes | `cargo install hermes` |

### B. 错误码映射

| Scanner 错误 | 扩展行为 |
|--------------|----------|
| `BINARY_NOT_FOUND` | 提示安装 od-cli-scanner，提供安装命令 |
| `TIMEOUT` | 显示 "检测超时"，提供重试按钮 |
| `PARSE_ERROR` | 显示 "结果解析失败"，记录日志 |

---

*此文档由 PM 老顾编写，遵循双语标准。版本控制：od-cli-scanner/docs/vscode-plugin-prd.md*
