# od-cli-scanner 代码审查报告

> 审查日期: 2026-06-22
> 项目路径: `~/projects/od-cli-scanner`
> 对比基准: `~/projects/open-design/apps/daemon/src/runtimes/` (TypeScript 原始实现)

---

## 一、P0 问题（阻塞性 / 安全风险）

### 1. `main.rs` 是 Hello World 占位符
**位置**: `src/main.rs:1-3`

```rust
fn main() {
    println!("Hello, world!");
}
```

**问题**: CLI 入口完全未实现，与 `cli/mod.rs` 中精心设计的 Clap 结构脱节。`od-scan` 二进制编译后无法执行任何扫描逻辑。

**建议**: 连接 `Cli` 解析到 `detect_agents` 调用，或删除 `[[bin]]` 声明直到实现完成。

---

### 2. Cargo.toml 中 napi 依赖配置错误
**位置**: `Cargo.toml:52-58`

```toml
# napi-rs (Node bindings)
[napi-dependencies]
api = { version = "2", features = ["napi8", "tokio_rt"] }
api-derive = "2"
```

**问题**: `[napi-dependencies]` 不是有效的 Cargo 配置节。正确应为 `[dependencies]` 下的 `napi` 和 `napi-derive`。

**建议**:
```toml
[dependencies]
# ... existing deps ...
napi = { version = "2", features = ["napi8", "tokio_rt"] }
napi-derive = "2"

[build-dependencies]
napi-build = "2"
```

---

### 3. `probe.rs` 错误码硬编码且跨平台不安全
**位置**: `src/core/probe.rs:34-44`

```rust
if code == Some(2) || code == Some(3) {
    // ENOENT / ESRCH
    Err(ProbeError::NotInvocable(NotInvocableCause::MissingTarget))
} else if code == Some(13) {
    // EACCES
    Err(ProbeError::NotInvocable(NotInvocableCause::NotExecutable))
}
```

**问题**:
- 硬编码 errno 值（2=ENOENT, 3=ESRCH, 13=EACCES）在 Windows 上完全不适用。Windows 使用 `ERROR_FILE_NOT_FOUND` (2) 但语义不同。
- `raw_os_error()` 在 Windows 上返回的是 Win32 错误码，不是 POSIX errno。
- 缺少 `ENOTDIR` 处理（TS 原始实现覆盖）。

**建议**: 使用 `std::io::ErrorKind` 匹配，而非硬编码 errno：

```rust
use std::io::ErrorKind;

match e.kind() {
    ErrorKind::NotFound | ErrorKind::NotADirectory => {
        Err(ProbeError::NotInvocable(NotInvocableCause::MissingTarget))
    }
    ErrorKind::PermissionDenied => {
        Err(ProbeError::NotInvocable(NotInvocableCause::NotExecutable))
    }
    _ => Ok(None),
}
```

---

### 4. `is_executable` 在 Windows 上存在严重缺陷
**位置**: `src/core/executables.rs:75-82`

```rust
#[cfg(windows)]
{
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    return matches!(ext.as_str(), "exe" | "cmd" | "bat" | "ps1");
}
```

**问题**:
- 未检查 `PATHEXT` 环境变量，而 TS 实现会动态读取 `PATHEXT`（`executables.ts:138`）。
- 仅检查扩展名，未验证文件是否真实存在、是否为常规文件。
- 缺少 `.COM`、`.VBS`、`.JS` 等常见可执行扩展名。
- 与 TS 实现相比，缺少 `looksExecutableOnWindows` 的完整逻辑。

---

### 5. 并发安全：无 per-probe fault isolation
**位置**: `src/core/detector.rs:15-20`

```rust
let futures: Vec<_> = defs
    .iter()
    .map(|def| detect_single_agent(def, configured_env))
    .collect();
let agents = join_all(futures).await;
```

**问题**: `join_all` 中任一 future panic 会导致整个扫描崩溃。TS 原始实现有 `safeProbe` 包装器（`detection.ts:289-305`），每个 agent 的探测被 `try/catch` 隔离，确保单个 agent 失败不会导致整个 picker 空白。

**建议**: 添加 `safe_detect_single_agent` 包装：

```rust
async fn safe_detect_single_agent(
    def: &AgentDef,
    configured_env: &AgentEnvConfig,
) -> DetectedAgent {
    match std::panic::AssertUnwindSafe(detect_single_agent(def, configured_env))
        .catch_unwind()
        .await
    {
        Ok(result) => result,
        Err(_) => make_unavailable(def, vec![AgentDiagnostic {
            kind: "probe_crash".into(),
            message: "Agent probe crashed unexpectedly".into(),
            fix_actions: Some(vec![FixAction { kind: "rescan".into(), label: None }]),
        }]),
    }
}
```

---

## 二、P1 问题（架构 / 功能缺失）

### 6. 缺少 Streaming 检测 API
**TS 原始实现**: `detection.ts:346-363` 提供 `detectAgentsStream`（AsyncGenerator），允许 UI 在 agent 探测完成时立即渲染，无需等待最慢 CLI。

**Rust 缺失**: 只有 `detect_agents` 批量 API。

**建议**: 添加 `detect_agents_stream` 返回 `impl Stream<Item = DetectedAgent>`，使用 `tokio::sync::mpsc` 或 `futures::channel`。

---

### 7. 缺少 Capability Probing
**TS 原始实现**: `detection.ts:167-189` 的 `probeCapabilities` 通过 `--help` 解析 CLI 支持的 flag，缓存到 `agentCapabilities` 供 `buildArgs` 使用。

**Rust 缺失**: `probe.rs` 完全没有 capability 探测逻辑。

**建议**: 在 `AgentDef` 中添加 `help_args` 和 `capability_flags` 字段，实现 `probe_capabilities` 函数。

---

### 8. 模型探测只有 Fallback，无 Live 实现
**位置**: `src/core/probe.rs:88-97`

```rust
pub async fn probe_models(
    _bin_path: &std::path::Path,
    _args: Option<&[String]>,
    _timeout_ms: u64,
    fallback: &[ModelOption],
) -> (Vec<ModelOption>, ModelsSource) {
    (fallback.to_vec(), ModelsSource::Fallback)
}
```

**TS 原始实现**: `detection.ts:49-88` 的 `fetchModels` 支持两种 live 模型获取：
- 通过 `def.fetchModels` 函数（AMR 特殊逻辑）
- 通过 `def.listModels` 执行 CLI 命令并解析 stdout（如 opencode 的 `models` 命令）
- 支持 `maxBuffer: 8MB` 处理大模型列表
- 支持 `rememberLiveModels` 缓存和 `getRememberedLiveModels` 回退

**建议**: 设计 `ModelProbe` trait：

```rust
#[async_trait]
pub trait ModelProbe: Send + Sync {
    async fn fetch_models(
        &self,
        bin_path: &Path,
        env: &HashMap<String, String>,
    ) -> Result<Vec<ModelOption>, ProbeError>;
}

pub struct CliModelProbe {
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub parser: Box<dyn Fn(&str) -> Vec<ModelOption> + Send + Sync>,
}
```

---

### 9. 缺少 Launch Resolution 层
**TS 原始实现**: `launch.ts:15-37` 的 `resolveAgentLaunch` 区分 `selectedPath`（显示给用户）和 `launchPath`（实际启动），并处理 Codex native binary 升级逻辑。

**Rust 缺失**: `executables.rs` 只有 `resolve_executable`，没有 launch 层。

**问题**: 对于 Codex under nvm/fnm/mise，PATH 可见的 shim 可能不可执行，但 launch resolver 可以升级到 packaged native binary。Rust 实现如果探测 shim 会错误报告 "not installed"。

---

### 10. 环境变量处理过于简单
**TS 原始实现**: `env.ts` 的 `spawnEnvForAgent` 处理：
- 系统代理环境变量（`HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`）
- 大小写不敏感 PATH 键（Windows `Path` vs `PATH`）
- Sandbox 模式根目录重定向
- AMR profile 解析和注入
- 继承/覆盖 API key（`ANTHROPIC_API_KEY` 等）

**Rust 缺失**: `AgentEnvConfig` 只是简单的 `HashMap<String, HashMap<String, String>>`，没有上述任何逻辑。

---

### 11. 诊断信息不完整
**TS 原始实现**: `diagnostics.ts` 提供丰富的诊断：
- `searchedDirs`：探测时实际搜索的目录列表
- `severity`: `error` / `warning`
- `reason`: `not-on-path`, `not-executable`, `auth-missing` 等
- `detail`: 额外上下文（如不可执行文件的具体路径）

**Rust 缺失**: `AgentDiagnostic` 结构缺少 `severity`、`reason`、`searchedDirs`、`detail` 字段。

---

### 12. 缺少 Agent 注册表和本地 Profile 支持
**TS 原始实现**: `registry.ts` 维护 24 个内置 agent 定义，支持 `readLocalAgentProfileDefs` 加载用户自定义 agent。

**Rust 缺失**: 没有内置 agent 定义列表，没有本地 profile 加载机制。`detect_agents` 要求调用者传入完整的 `AgentDef` 列表。

---

## 三、P2 问题（改进建议）

### 13. Windows 平台差异处理建议

当前代码的 Windows 支持是碎片化的。建议引入平台抽象层：

```rust
// src/platform/mod.rs
pub trait Platform {
    fn path_delimiter(&self) -> &'static str;
    fn path_exts(&self) -> Vec<String>;
    fn is_executable(&self, path: &Path) -> bool;
    fn resolve_on_path(&self, bin: &str, dirs: &[PathBuf]) -> Option<PathBuf>;
    fn case_insensitive_env_key<'a>(&self, env: &'a HashMap<String, String>, key: &str) -> Option<&'a String>;
}

pub struct UnixPlatform;
pub struct WindowsPlatform;
```

关键改进点：
- **PATH 大小写**: Windows 环境变量键大小写不敏感（`Path` vs `PATH`），`applyAgentLaunchEnv` 需要查找实际键名。
- **PATHEXT**: Windows 可执行扩展名应从 `PATHEXT` 环境变量读取，而非硬编码。
- **路径分隔符**: 使用 `std::path::MAIN_SEPARATOR` 而非硬编码 `/` 或 `\`。
- **权限模型**: Windows 没有 Unix 的 `mode & 0o111`，应使用 `AccessCheck` 或至少检查扩展名 + 文件存在性。

---

### 14. Live Model Probing 架构建议

参考 TS 实现的 `fetchModels` 和 `models.ts`，建议设计如下：

```rust
// src/core/models.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModelCache {
    cache: Arc<RwLock<HashMap<String, Vec<ModelOption>>>>,
}

impl ModelCache {
    pub async fn remember(&self, agent_id: &str, models: &[ModelOption], scope: Option<&str>) {
        let key = format!("{}\0{}", agent_id, scope.unwrap_or(""));
        self.cache.write().await.insert(key, models.to_vec());
    }
    
    pub async fn get_remembered(&self, agent_id: &str, scope: Option<&str>) -> Vec<ModelOption> {
        let key = format!("{}\0{}", agent_id, scope.unwrap_or(""));
        self.cache.read().await.get(&key).cloned().unwrap_or_default()
    }
}

#[async_trait]
pub trait ModelParser: Send + Sync {
    fn parse(&self, stdout: &str) -> Vec<ModelOption>;
}

pub struct LiveModelProber {
    cache: ModelCache,
}

impl LiveModelProber {
    pub async fn probe(
        &self,
        def: &AgentDef,
        bin_path: &Path,
        env: &HashMap<String, String>,
    ) -> (Vec<ModelOption>, ModelsSource) {
        // 1. Try fetchModels function if available
        if let Some(ref fetcher) = def.fetch_models {
            match fetcher.fetch(bin_path, env).await {
                Ok(models) if !models.is_empty() => {
                    self.cache.remember(&def.id, &models, None).await;
                    return (models, ModelsSource::Live);
                }
                _ => {}
            }
        }
        
        // 2. Try listModels CLI command
        if let Some(ref list_models) = def.list_models_args {
            match probe_version(bin_path, list_models, def.list_models_timeout_ms.unwrap_or(5000)).await {
                Ok(Some(stdout)) => {
                    if let Some(ref parser) = def.model_parser {
                        let models = parser.parse(&stdout);
                        if !models.is_empty() {
                            self.cache.remember(&def.id, &models, None).await;
                            return (models, ModelsSource::Live);
                        }
                    }
                }
                _ => {}
            }
        }
        
        // 3. Fall back to remembered models
        let remembered = self.cache.get_remembered(&def.id, None).await;
        if !remembered.is_empty() {
            return (remembered, ModelsSource::Live);
        }
        
        // 4. Static fallback
        (def.fallback_models.clone(), ModelsSource::Fallback)
    }
}
```

---

### 15. napi-rs 绑定层设计建议

当前 Cargo.toml 配置错误，且没有 napi 绑定代码。建议设计如下：

```rust
// src/napi/mod.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct JsDetectedAgent {
    pub id: String,
    pub name: String,
    pub bin: String,
    pub available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub models: Vec<JsModelOption>,
    pub models_source: String,
    pub auth_status: Option<String>,
    pub diagnostics: Option<Vec<JsAgentDiagnostic>>,
}

#[napi]
pub async fn detect_agents_js(
    config_json: Option<String>,
) -> Result<Vec<JsDetectedAgent>> {
    let defs: Vec<AgentDef> = if let Some(json) = config_json {
        serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))?
    } else {
        // Load built-in defs
        default_agent_defs()
    };
    
    let result = crate::core::detector::detect_agents(&defs, &Default::default()).await;
    
    Ok(result.agents.into_iter().map(into_js).collect())
}

#[napi]
pub fn detect_agents_stream_js(
    env: Env,
    config_json: Option<String>,
) -> Result<JsObject> {
    // Return a Readable stream or AsyncIterator
    // Use napi-rs ThreadSafeFunction for callback-based streaming
}
```

关键考虑：
- **错误转换**: Rust `thiserror` 错误需要转换为 napi `Error::from_reason`
- **大缓冲区**: model listing 可能超过 1MB（如 openrouter），需要 `maxBuffer` 等价物
- **流式 API**: 使用 `tokio::sync::mpsc` + `ThreadSafeFunction` 实现增量返回
- **跨平台**: napi-rs 的 `tokio_rt` feature 已启用，但需要确保 tokio runtime 在 Node 线程中正确初始化

---

## 四、与 TypeScript 原始实现的对比总结

| 功能 | TS 实现 | Rust 实现 | 差距 |
|------|---------|-----------|------|
| 内置 Agent 注册表 | 24 个 agent，支持本地 profile | 无 | **重大缺失** |
| 可执行文件解析 | `resolveOnPath` + `inspectAgentExecutableResolution` + `resolveAgentLaunch` | 简单的 `resolve_executable` | 缺少 launch 层、Codex native 升级 |
| 版本探测 | `probeVersionAtPath`，区分 not-invocable / spawned | `probe_version`，硬编码 errno | Windows 不安全 |
| 模型探测 | Live `fetchModels` / `listModels` + `rememberLiveModels` | 只有 fallback | **重大缺失** |
| 认证探测 | `probeAgentAuthStatus` | `probe_auth`（简化版） | 缺少 auth message |
| Capability 探测 | `probeCapabilities` + 缓存 | 无 | **缺失** |
| 环境处理 | `spawnEnvForAgent`，代理、sandbox、大小写 PATH | 简单的 HashMap | **重大缺失** |
| 流式检测 | `detectAgentsStream` (AsyncGenerator) | 无 | **缺失** |
| 故障隔离 | `safeProbe` catch-all | 无 | **安全风险** |
| 诊断系统 | 丰富的 `reason`, `severity`, `searchedDirs`, `fixActions` | 简化版 | 信息不足 |
| Windows 支持 | `PATHEXT`, 大小写 PATH, 完整可执行检查 | 硬编码扩展名 | **不安全** |
| 打包构建支持 | `OD_RESOURCE_ROOT`, `packagedBuiltInExecutable` | 无 | **缺失** |

---

## 五、优先改进路线图

### 阶段 1: 修复 P0（立即）
1. 修复 `main.rs` 连接 CLI 或移除 bin 声明
2. 修复 `Cargo.toml` napi 依赖配置
3. 替换 `probe.rs` 中的硬编码 errno 为 `ErrorKind`
4. 修复 `is_executable` Windows 实现，读取 `PATHEXT`
5. 添加 `safe_detect_single_agent` 故障隔离

### 阶段 2: 补齐核心功能（1-2 周）
6. 添加内置 Agent 注册表（参考 `registry.ts`）
7. 实现 `resolveAgentLaunch` 层（区分 selected/launch path）
8. 实现 Live Model Probing（`fetchModels` + `listModels` + 缓存）
9. 实现 Capability Probing
10. 添加 Streaming API (`detect_agents_stream`)

### 阶段 3: 平台与绑定（2-3 周）
11. 引入平台抽象层，完整 Windows 支持
12. 实现 `spawnEnvForAgent` 等价物（代理、sandbox、PATH 处理）
13. 实现 napi-rs 绑定层
14. 添加测试覆盖（参考 `detection-resilience.test.ts` 等）

---

*报告结束*
