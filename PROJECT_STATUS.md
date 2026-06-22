# od-cli-scanner 项目进度表

> 最后更新: 2026-06-22 22:35 (Asia/Shanghai)
> 项目地址: https://github.com/gushuaialan1/od-cli-scanner

---

## 项目概况

| 项目 | od-cli-scanner |
|------|----------------|
| 描述 | Rust-native CLI agent detection engine — 检测 24 个 AI coding agents |
| 版本 | v0.1.0 |
| 状态 | **已发布** (tag v0.1.0) |
| 分支 | master |
| 最新提交 | `e5322d9` ci: fix napi build output path |

---

## 已完成 (Done)

| # | 任务 | 提交 | 负责人 |
|---|------|------|--------|
| 1 | 项目骨架 (cargo workspace, 3 crates) | `c83121b` | CEO |
| 2 | 核心检测引擎 (types/executables/probe/detector) | `c83121b` | Coder |
| 3 | CLI 二进制 (od-scan, clap, 3 output formats) | `c83121b` | Coder |
| 4 | napi-rs Node.js 绑定 | `c83121b` | Coder |
| 5 | npm 包结构 (JS/TS 封装 + 预编译二进制) | `c83121b` | Coder |
| 6 | 双语 README (Hero + Badges + Quick Start + API) | `c83121b` | CEO |
| 7 | 24 个内置 Agent 定义 | `c83121b` | Coder |
| 8 | 跨平台 errno 修复 (ErrorKind) | `c83121b` | Coder |
| 9 | Windows PATHEXT 支持 | `c83121b` | Coder |
| 10 | kimi/hermes 超时调整 (3s→10s) | `c83121b` | CEO |
| 11 | GitHub Actions CI (check/fmt/clippy/napi) | `845fd14` | CEO |
| 12 | GitHub Actions Release (CLI + napi + publish) | `845fd14` | CEO |
| 13 | README 致谢 Open Design | `29652c1` | CEO |
| 14 | 清理开发文档 (REVIEW.md, docs/) | `b1aa9d4` | CEO |
| 15 | 修复 napi build 输出路径 | `e5322d9` | CEO |

---

## 当前状态 (In Progress)

| # | 任务 | 状态 | 阻塞项 | 下一步 |
|---|------|------|--------|--------|
| 16 | **GitHub Actions Release 构建** | 🟡 运行中 | 等待构建结果 | 验证 5 平台 CLI + napi 产物 |

---

## 待办 (Todo)

| # | 任务 | 优先级 | 说明 |
|---|------|--------|------|
| 17 | 配置 Secrets (NPM_TOKEN, CARGO_TOKEN) | P0 | 在 GitHub Settings → Secrets 添加 |
| 18 | 验证 Release 产物下载 | P0 | 确认 CLI 二进制和 napi 模块可下载 |
| 19 | 补充单元测试 | P1 | 当前测试覆盖率不足 |
| 20 | Live model probing | P1 | 从 fallback 升级到实时探测 |
| 21 | Capability flag detection | P1 | `--help` 探测 |
| 22 | Streaming detection results | P2 | SSE/WebSocket |
| 23 | WASM target | P2 | 浏览器支持 |

---

## 已知问题 (Known Issues)

| # | 问题 | 状态 | 备注 |
|---|------|------|------|
| 1 | GitHub Actions napi build 路径错误 | ✅ 已修复 | `e5322d9` 添加 `-o ../../npm` |
| 2 | 24 Agent 并发探测超时 | ✅ 已缓解 | kimi/hermes 10s 超时 |
| 3 | 无单元测试 | ⏳ 待办 | 需要补充 |

---

## 技术栈

| 层级 | 技术 |
|------|------|
| 核心 | Rust (tokio, serde, clap, which, futures) |
| 绑定 | napi-rs (Node.js) |
| 构建 | cargo, cross (交叉编译) |
| CI/CD | GitHub Actions |
| 包管理 | crates.io + npm |

---

## 快速命令

```bash
# 本地构建
cd ~/projects/od-cli-scanner
cargo build --release

# 运行扫描
./target/release/od-scan --format table

# 测试 napi
napi build --platform --release --package od-cli-scanner-napi

# 提交并推送
git add -A && git commit -m "..."
git push origin master

# 打 tag 触发 Release
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin v0.1.1
```

---

## 参考项目

- **Open Design**: https://github.com/nexu-io/open-design
- 致谢: 本项目 Agent 定义和探测策略源自 Open Design 的 TypeScript 运行时注册表

---

## 维护者

- CEO: 老顾
- Coder: AI subagent (delegate_task)

---

*此文件由 CEO 维护，任何 agent 接手前请先阅读。*
