# 参考研究：open-design mock CLI 测试基础设施 → od-cli-scanner 集成测试方案

> 研究员报告 · 2026-07-29
> 参考项目：`~/projects/open-design`（mocks/ + apps/daemon 测试体系）
> 目标项目：`~/projects/od-cli-scanner`（Rust workspace: od-cli-scanner 核心 / od-scan CLI / napi 绑定 + packages/vscode 扩展）

---

## 1. open-design mock CLI 架构核心要点

### 实现方式（`mocks/`，纯 Node ≥18，零依赖）
- **入口**：`mock-agent.mjs` —— `node mock-agent.mjs --as <agent>`，按 agent 路由到对应 format renderer。
- **PATH-overlay**：`mocks/bin/` 下 19 个同名 bash wrapper（`claude`/`codex`/`gemini`/`opencode`/…），每个 wrapper 只做一件事：
  ```bash
  exec node "$HERE/../mock-agent.mjs" --as claude "$@"
  ```
  测试时 `export PATH="$PWD/mocks/bin:$PATH"`，被测进程 spawn `claude` 时拿到的是 mock，不是真实 CLI。
- **回放数据源**：`mocks/recordings/<trace-id>.jsonl`（Langfuse 匿名化 trace，179 条），**不进 git**；`mocks/manifest.json`（已提交，含 trace_id/sha256/agent/outcome/skills/provenance）是 source of truth，`scripts/fetch-recordings.sh` 从 R2 按需拉取并 sha256 校验。
- **录制选择**（env 驱动，优先级递减）：`OD_MOCKS_TRACE=<id>` 固定回放；`OD_MOCKS_BY_PROMPT_HASH=1` 按 prompt 哈希确定性选取；`OD_MOCKS_POOL=agent:claude` 池内随机；`OD_MOCKS_SEED` 可复现随机；`OD_MOCKS_NO_DELAY=1` 跳过事件间隔。
- **协议保真**：每个 renderer（`lib/format-claude.mjs` 等）逐行对齐 daemon 解析器（`claude-stream.ts` / `json-event-stream.ts`），分三类：stdout 流式 JSON、纯文本 stdout、ACP JSON-RPC over stdio（含 `vela` 的 set_model 门控、login/models 子命令）。
- **Golden 快照**：`mocks/golden/<trace>.events.json` 提交入 git；`apps/daemon/tests/mocks-golden.test.ts` 用真实 parser 跑 mock 输出并 diff 事件序列。`MOCKS_GOLDEN_UPDATE=1` 再生成；易变字段（sessionId）归一化为 `<normalized>`。抓 parser 语义回归，不只是崩溃。
- **Contract check**：`scripts/contract-check.sh` 并排跑真实 CLI vs mock，对比事件类型分布——防录制本身与真实 CLI 漂移（人工触发，烧 token，不进 CI）。

### daemon runtime 测试方式（`apps/daemon/tests/runtimes/`）
- **可执行文件发现**（`src/runtimes/executables.ts`）：支持 per-agent env 覆盖（`CLAUDE_BIN` 等）、`fallbackBins`（如 openclaude）、`OD_AGENT_HOME` 沙箱 home（设置后跳过系统 bin 目录并清空 env，保证测试与宿主机环境无关）、well-known toolchain bin 目录缓存（5s TTL）。
- **测试模式**（`tests/runtimes/executables.test.ts` / `env-and-detection.test.ts`）：
  - `mkdtempSync` 建临时 home，写入 `#!/bin/sh\nexit 0` 假二进制 + `chmod 0o755`；
  - `withEnvSnapshot(['PATH', ...])` helper 保存/恢复 env；
  - `withPlatform` mock 平台分支；Windows 下跳过文件系统用例（PATHEXT 差异），但声明式断言（fallbackBins 数组内容）全平台跑。
- **终端启动**（`terminal-launch.ts`）：macOS osascript / Linux x-terminal-emulator 链 / 不支持平台返回结构化 `{ ok: false, reason }`；测试只 pin 不支持平台的返回 shape，不真开终端。
- **node-pty 自愈**：`terminals.spawn-helper.test.ts` 复现 pnpm 解包丢 +x 位的真实 bug，`create()` 时自动修复。

### CI（`.github/workflows/ci.yml`，962 行）
关键 job：`scopes`（变更范围路由）→ `static_gate`（actionlint 等）→ `preflight`（guard + i18n）→ `workspace_unit_tests`（daemon build + vitest）→ `nix_validation` / `web_workspace_tests` / `windows_tools_pack_payload_tests`。daemon 测试若依赖录制语料，前置跑 `fetch-recordings.sh` 并缓存 `mocks/recordings/`。

---

## 2. od-cli-scanner 可直接借鉴的 5 个点

1. **PATH-overlay + bash wrapper 是 mock CLI 的最小正确抽象**：不需要容器、不需要 patch 被测代码。od-cli-scanner 的 detector 通过 PATH 找二进制，测试时只需把 `mocks/bin/` 前置到 PATH。wrapper 可退化为一行 `#!/bin/sh` + 固定输出（比 open-design 简单得多——我们只需 replay `--version` / `--help` 的静态输出，不需要流式协议）。
2. **env 快照 helper 隔离测试**：`withEnvSnapshot` 模式（Rust 里用 RAII guard 结构体实现 Drop 恢复 env），配合 `OD_SCANNER_HOME` 式的 home 覆盖，让检测测试与宿主机安装的 CLI 完全解耦。open-design 特别强调：不设 home override 时测试结果是机器相关的——od-cli-scanner 现有 `executables_test.rs` 应统一走这个模式。
3. **结构化失败 shape 用测试 pin 住**：terminal-launch 测试只断言 `{ ok: false, reason }` 的 shape 而非真开终端。对应到 `terminalLauncher.ts` / `spawn.rs`：注入 `Platform` 参数，测试传假平台，断言结构化错误。
4. **Golden 快照防语义回归**：mock 输出 → 真实 parser → 事件序列快照。对应 od-cli-scanner：固定 mock CLI 的 `--version` 输出 → `version_parser.rs` → 解析结果 JSON 快照（Rust 侧用 `insta` crate 是社区标准，等价于 golden files）。更新流程 `INSTA_UPDATE=1 cargo test` + 人工 review diff。
5. **fixture 不进 git / manifest 进 git 的分离**：od-cli-scanner 不需要 R2 语料库（我们录的是静态 version 字符串，几十字节），但要学它的原则：**fixture 清单（哪些 CLI、什么输出样例、对应期望解析结果）以 manifest JSON 提交**，测试代码读 manifest 驱动，新增 CLI 样例 = 改 manifest 不改代码。

---

## 3. od-cli-scanner 集成测试方案建议

### 分层

| 层 | 测什么 | 工具 | mock 位置 |
|---|---|---|---|
| Rust 核心 | detector/executables/probe/version_parser | `cargo test` + `assert_cmd` + `predicates` + `insta`（快照）| `crates/od-cli-scanner/tests/fixtures/bin/` |
| od-scan CLI | argv 解析、table/json/csv 输出端到端 | `assert_cmd` 跑真实编译产物 | 同上，PATH 注入 |
| napi 绑定 | Node 侧调用 `scan()` 返回结构 | node:test 或 vitest，`require()` 编译产物 | 同上 |
| VS Code 扩展 | agentService/scannerBridge/tree provider/terminalLauncher | `@vscode/test-electron`（集成）+ vitest（纯逻辑单测）| `packages/vscode/test-fixtures/bin/`（或复用 Rust 侧 fixture） |

### Rust 核心集成测试（重点，当前缺口最大）
- **mock CLI 实现**：`tests/fixtures/bin/<agent>` 是 shell 脚本（Windows 下配套 `.cmd`/`.ps1`），响应 `--version`/`--help`/`auth status` 打印 fixture 输出后 `exit 0`。远比 open-design 简单：无流式协议、无 JSONL 语料，输出直接内嵌脚本或读同目录 `.txt`。
- **测试 harness**（仿 `withEnvSnapshot` + `mkdtemp`）：
  ```rust
  struct EnvGuard { saved: Vec<(String, Option<String>)> }  // Drop 时恢复
  fn with_fake_path(agents: &[&str]) -> (TempDir, EnvGuard) // 建临时 bin 目录，软链/复制 mock，重写 PATH
  ```
- **场景矩阵**：① PATH 上只有 mock → 检出 N 个 agent，version 正确解析；② 空 PATH → 全部 undetected；③ mock 超时（`sleep 10`）→ Timeout 错误（现有 probe_test 已有模式，扩展到 detector 层）；④ mock 非零退出 → `NotInvocableCause` 分类正确；⑤ 同名二进制非可执行（0644）→ 跳过不误报。
- **快照**：`version_parser` 对每个 agent 的 3-5 个真实版本输出样例跑 `insta::assert_json_snapshot!`，manifest 驱动。
- **并发安全**：Rust 测试默认并行，env 修改是进程全局——PATH 类用例集中放一个 `integration` 测试 binary 并用 `serial_test` crate 串行，或改用子进程跑 `od-scan`（assert_cmd 天然隔离 env）。

### VS Code 扩展集成测试
- **纯逻辑单测**（vitest，无需 Electron）：`versionParser` 复用、`agentTreeProvider` 的分组/排序、`commandController` 的状态机——mock `scannerBridge` 接口注入固定 `ScanResult`。
- **Electron 集成**（`@vscode/test-electron`）：扩展激活 → 调用真实 napi `scan()`（PATH 注入 fixture bin）→ 断言 tree 渲染和 terminal 启动命令拼接。terminalLauncher 注入假 platform 断言结构化失败（借鉴点 3）。
- **fixture 复用**：`packages/vscode/test-fixtures/bin` 只做 symlink/构建期拷贝指向 `crates/od-cli-scanner/tests/fixtures/bin`，单一 source of truth。

---

## 4. CI pipeline 建议（在现有 check/fmt/clippy/build-napi 基础上增量）

```yaml
# 新增到 .github/workflows/ci.yml
rust-integration:              # matrix: ubuntu / macos / windows
  - cargo test --workspace --all-features
  - cargo insta test --check   # 快照未提交则失败
cli-e2e:                       # 仅 ubuntu，快
  - cargo build -p od-scan
  - PATH="tests/fixtures/bin:$PATH" ./target/debug/od-scan --format json | jq .   # 断言全检出
vscode-test:                   # xvfb-run，ubuntu only 即可
  - cd packages/vscode && npm ci && npm run compile
  - xvfb-run -a npx @vscode/test-electron --extensionTestsPath=out/test
```

要点：
- 沿用现有 matrix OS 结构；fixture shell 脚本在 Windows job 上自动走 `.cmd` 变体（open-design 的做法是 win32 跳过 fs 用例、保留声明式断言——od-cli-scanner 已有 `#[cfg(unix)]` 先例，可继续）。
- 快照检查与 fmt/clippy 同级 fail-fast；更新快照只在本地 `INSTA_UPDATE=1` 后提交 diff。
- 不需要 open-design 的 R2/fetch 步骤——我们的 fixture 总量是 KB 级，直接进 git。

---

## 附：研究过程中确认的文件

- `mocks/README.md`（482 行，完整读完）、`mock-agent.mjs`、`mocks/bin/{claude,codex}` wrapper、`mocks/golden/README.md`
- `apps/daemon/src/runtimes/executables.ts`、`terminal-launch.ts`
- `apps/daemon/tests/`：`mocks-golden.test.ts`、`terminals.spawn-helper.test.ts`、`runtimes/executables.test.ts`、`runtimes/env-and-detection.test.ts`、`runtimes/terminal-launch.test.ts`
- `.github/workflows/ci.yml` job 结构
- od-cli-scanner 现状：`crates/od-cli-scanner/tests/{executables_test,probe_test}.rs`（已有 probe 单测基础）、`packages/vscode/src/`（10 个 TS 文件，无测试）
