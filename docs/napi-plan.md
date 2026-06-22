# napi-rs Binding & npm Packaging Plan

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              npm Package                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  TypeScript API (index.ts)                                        │   │
│  │  - Scanner class                                                    │   │
│  │  - scanAgents(): Promise<DetectionResult>                           │   │
│  │  - getAgentDefs(): AgentDef[]                                       │   │
│  │  - checkBinary(): Promise<DetectedAgent | null>                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  napi-rs Native Module (.node)                                      │   │
│  │  - Prebuilt binaries for all platforms                            │   │
│  │  - Fallback: build from source                                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Rust Core Engine (od-cli-scanner crate)                            │   │
│  │  - detect_agents()                                                  │   │
│  │  - AgentDef, DetectedAgent, DetectionResult                         │   │
│  │  - probe, executables, types                                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 2. napi-rs Binding Architecture

### 2.1 Rust Side (napi-rs exports)

File: `crates/napi/src/lib.rs`

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;
use od_cli_scanner::core::types::*;
use od_cli_scanner::core::detector::detect_agents;
use std::collections::HashMap;

// ─── Re-export types with napi derive ───────────────────────────────

#[napi(object)]
pub struct JsModelOption {
    pub id: String,
    pub label: String,
}

#[napi(string_enum)]
pub enum JsAuthStatus {
    Ok,
    Missing,
    Unknown,
}

#[napi(string_enum)]
pub enum JsModelsSource {
    Live,
    Fallback,
}

#[napi(object)]
pub struct JsAgentDiagnostic {
    pub kind: String,
    pub message: String,
    pub fix_actions: Option<Vec<JsFixAction>>,
}

#[napi(object)]
pub struct JsFixAction {
    pub kind: String,
    pub label: Option<String>,
}

#[napi(object)]
pub struct JsDetectedAgent {
    pub id: String,
    pub name: String,
    pub bin: String,
    pub available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub models: Vec<JsModelOption>,
    pub models_source: JsModelsSource,
    pub auth_status: Option<JsAuthStatus>,
    pub auth_message: Option<String>,
    pub diagnostics: Option<Vec<JsAgentDiagnostic>>,
    pub stream_format: Option<String>,
    pub install_url: Option<String>,
    pub docs_url: Option<String>,
}

#[napi(object)]
pub struct JsDetectionResult {
    pub agents: Vec<JsDetectedAgent>,
    pub scanned_at: String,
    pub duration_ms: u64,
}

#[napi(object)]
pub struct JsAgentDef {
    pub id: String,
    pub name: String,
    pub bin: String,
    pub fallback_bins: Vec<String>,
    pub version_args: Vec<String>,
    pub version_probe_timeout_ms: u64,
    pub fallback_models: Vec<JsModelOption>,
    pub stream_format: String,
    pub install_url: Option<String>,
    pub docs_url: Option<String>,
    pub bin_env_key: Option<String>,
    pub auth_probe_args: Option<Vec<String>>,
    pub auth_probe_timeout_ms: Option<u64>,
    pub list_models_args: Option<Vec<String>>,
    pub list_models_timeout_ms: Option<u64>,
}

// ─── Conversion helpers ──────────────────────────────────────────────

impl From<ModelOption> for JsModelOption {
    fn from(m: ModelOption) -> Self {
        Self { id: m.id, label: m.label }
    }
}

impl From<AuthStatus> for JsAuthStatus {
    fn from(a: AuthStatus) -> Self {
        match a {
            AuthStatus::Ok => JsAuthStatus::Ok,
            AuthStatus::Missing => JsAuthStatus::Missing,
            AuthStatus::Unknown => JsAuthStatus::Unknown,
        }
    }
}

impl From<ModelsSource> for JsModelsSource {
    fn from(m: ModelsSource) -> Self {
        match m {
            ModelsSource::Live => JsModelsSource::Live,
            ModelsSource::Fallback => JsModelsSource::Fallback,
        }
    }
}

impl From<FixAction> for JsFixAction {
    fn from(f: FixAction) -> Self {
        Self { kind: f.kind, label: f.label }
    }
}

impl From<AgentDiagnostic> for JsAgentDiagnostic {
    fn from(d: AgentDiagnostic) -> Self {
        Self {
            kind: d.kind,
            message: d.message,
            fix_actions: d.fix_actions.map(|v| v.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<DetectedAgent> for JsDetectedAgent {
    fn from(a: DetectedAgent) -> Self {
        Self {
            id: a.id,
            name: a.name,
            bin: a.bin,
            available: a.available,
            path: a.path,
            version: a.version,
            models: a.models.into_iter().map(Into::into).collect(),
            models_source: a.models_source.into(),
            auth_status: a.auth_status.map(Into::into),
            auth_message: a.auth_message,
            diagnostics: a.diagnostics.map(|v| v.into_iter().map(Into::into).collect()),
            stream_format: a.stream_format,
            install_url: a.install_url,
            docs_url: a.docs_url,
        }
    }
}

impl From<DetectionResult> for JsDetectionResult {
    fn from(r: DetectionResult) -> Self {
        Self {
            agents: r.agents.into_iter().map(Into::into).collect(),
            scanned_at: r.scanned_at,
            duration_ms: r.duration_ms,
        }
    }
}

impl From<AgentDef> for JsAgentDef {
    fn from(d: AgentDef) -> Self {
        Self {
            id: d.id,
            name: d.name,
            bin: d.bin,
            fallback_bins: d.fallback_bins,
            version_args: d.version_args,
            version_probe_timeout_ms: d.version_probe_timeout_ms,
            fallback_models: d.fallback_models.into_iter().map(Into::into).collect(),
            stream_format: d.stream_format,
            install_url: d.install_url,
            docs_url: d.docs_url,
            bin_env_key: d.bin_env_key,
            auth_probe_args: d.auth_probe_args,
            auth_probe_timeout_ms: d.auth_probe_timeout_ms,
            list_models_args: d.list_models_args,
            list_models_timeout_ms: d.list_models_timeout_ms,
        }
    }
}

// ─── napi exported functions ────────────────────────────────────────

/// Scan for all configured agents
#[napi]
pub async fn scan_agents(
    defs: Vec<JsAgentDef>,
    env_config: Option<HashMap<String, HashMap<String, String>>>,
) -> Result<JsDetectionResult> {
    let rust_defs: Vec<AgentDef> = defs.into_iter().map(|d| AgentDef {
        id: d.id,
        name: d.name,
        bin: d.bin,
        fallback_bins: d.fallback_bins,
        version_args: d.version_args,
        version_probe_timeout_ms: d.version_probe_timeout_ms,
        fallback_models: d.fallback_models.into_iter().map(|m| ModelOption {
            id: m.id,
            label: m.label,
        }).collect(),
        stream_format: d.stream_format,
        install_url: d.install_url,
        docs_url: d.docs_url,
        bin_env_key: d.bin_env_key,
        auth_probe_args: d.auth_probe_args,
        auth_probe_timeout_ms: d.auth_probe_timeout_ms,
        list_models_args: d.list_models_args,
        list_models_timeout_ms: d.list_models_timeout_ms,
    }).collect();

    let env = env_config.unwrap_or_default();
    let result = detect_agents(&rust_defs, &env).await;
    Ok(result.into())
}

/// Check a single binary (simplified API)
#[napi]
pub async fn check_binary(
    bin: String,
    version_args: Vec<String>,
    timeout_ms: Option<u64>,
) -> Result<Option<String>> {
    use od_cli_scanner::core::probe::probe_version;
    use std::path::PathBuf;

    let path = PathBuf::from(&bin);
    let timeout = timeout_ms.unwrap_or(3000);
    let result = probe_version(&path, &version_args, timeout).await;

    match result {
        Ok(v) => Ok(v),
        Err(_) => Ok(None),
    }
}
```

### 2.2 JS/TS Side (npm package)

File: `npm/index.ts`

```typescript
import {
  scanAgents as nativeScanAgents,
  checkBinary as nativeCheckBinary,
  JsAgentDef,
  JsDetectionResult,
  JsDetectedAgent,
} from './native';

export type {
  JsAgentDef as AgentDef,
  JsDetectionResult as DetectionResult,
  JsDetectedAgent as DetectedAgent,
  JsModelOption as ModelOption,
  JsAuthStatus as AuthStatus,
  JsModelsSource as ModelsSource,
  JsAgentDiagnostic as AgentDiagnostic,
  JsFixAction as FixAction,
} from './native';

// Built-in agent definitions (sync with Rust defaults)
export const DEFAULT_AGENT_DEFS: JsAgentDef[] = [
  {
    id: 'claude',
    name: 'Claude Code',
    bin: 'claude',
    fallback_bins: ['claude-code'],
    version_args: ['--version'],
    version_probe_timeout_ms: 3000,
    fallback_models: [
      { id: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4' },
      { id: 'claude-opus-4', label: 'Claude Opus 4' },
    ],
    stream_format: 'anthropic',
    install_url: 'https://docs.anthropic.com/claude-code',
    docs_url: 'https://docs.anthropic.com/claude-code',
    bin_env_key: 'CLAUDE_BIN',
    auth_probe_args: ['auth', 'status'],
    auth_probe_timeout_ms: 5000,
    list_models_args: ['models'],
    list_models_timeout_ms: 5000,
  },
  // ... more agents
];

export interface ScannerOptions {
  /** Custom agent definitions (replaces defaults if provided) */
  agentDefs?: JsAgentDef[];
  /** Per-agent environment overrides */
  envConfig?: Record<string, Record<string, string>>;
  /** Only return available agents */
  availableOnly?: boolean;
  /** Filter by agent IDs */
  filter?: string[];
}

/**
 * Scan for installed AI coding agents.
 * @returns Promise<DetectionResult>
 */
export async function scanAgents(options: ScannerOptions = {}): Promise<JsDetectionResult> {
  const defs = options.agentDefs ?? DEFAULT_AGENT_DEFS;
  const env = options.envConfig ?? {};

  let result = await nativeScanAgents(defs, env);

  if (options.availableOnly) {
    result = {
      ...result,
      agents: result.agents.filter(a => a.available),
    };
  }

  if (options.filter && options.filter.length > 0) {
    const filterSet = new Set(options.filter);
    result = {
      ...result,
      agents: result.agents.filter(a => filterSet.has(a.id)),
    };
  }

  return result;
}

/**
 * Check if a specific binary is available and get its version.
 * @returns Promise<string | null> version string or null
 */
export async function checkBinary(
  bin: string,
  versionArgs: string[] = ['--version'],
  timeoutMs?: number
): Promise<string | null> {
  return nativeCheckBinary(bin, versionArgs, timeoutMs);
}

// Re-export native types
export { nativeScanAgents, nativeCheckBinary };
```

## 3. npm Package Structure

```
od-cli-scanner/
├── package.json          # Package manifest
├── index.js              # Main entry (CJS/ESM loader)
├── index.d.ts            # TypeScript declarations
├── native.d.ts           # Auto-generated napi types
├── native.js             # Auto-generated napi loader
├── prebuilds/            # Prebuilt native binaries
│   ├── darwin-x64/
│   │   └── od-cli-scanner.node
│   ├── darwin-arm64/
│   │   └── od-cli-scanner.node
│   ├── linux-x64/
│   │   └── od-cli-scanner.node
│   ├── linux-arm64/
│   │   └── od-cli-scanner.node
│   ├── win32-x64/
│   │   └── od-cli-scanner.node
│   └── ...
├── src/                  # TypeScript source (optional, for custom JS wrapper)
│   └── index.ts
├── binding.gyp           # Fallback node-gyp build
└── README.md
```

### 3.1 package.json

```json
{
  "name": "od-cli-scanner",
  "version": "0.1.0",
  "description": "Node.js bindings for od-cli-scanner — detect installed AI coding agents",
  "main": "index.js",
  "types": "index.d.ts",
  "files": [
    "index.js",
    "index.d.ts",
    "native.js",
    "native.d.ts",
    "prebuilds"
  ],
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "prebuild": "prebuildify --napi --strip",
    "test": "node test.js"
  },
  "napi": {
    "name": "od-cli-scanner",
    "triples": {
      "defaults": false,
      "additional": [
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc"
      ]
    }
  },
  "engines": {
    "node": ">=16"
  },
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/nexu-io/od-cli-scanner.git"
  },
  "keywords": [
    "cli",
    "agent",
    "scanner",
    "ai",
    "detection",
    "napi-rs"
  ],
  "devDependencies": {
    "@napi-rs/cli": "^3.0.0",
    "prebuildify": "^6.0.0"
  }
}
```

### 3.2 Native Module Loader (native.js)

```javascript
const { existsSync, readFileSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;
const nativePackage = require('./package.json');
const { name } = nativePackage;

function loadNative() {
  // 1. Try prebuilt binary
  const prebuildPath = join(__dirname, 'prebuilds', `${platform}-${arch}`, `${name}.node`);
  if (existsSync(prebuildPath)) {
    return require(prebuildPath);
  }

  // 2. Try napi-rs build output (development)
  const buildPath = join(__dirname, `${name}.node`);
  if (existsSync(buildPath)) {
    return require(buildPath);
  }

  // 3. Fallback: try to find any .node file
  const fallbackPaths = [
    join(__dirname, `${name}.node`),
    join(__dirname, 'build', 'Release', `${name}.node`),
  ];
  for (const p of fallbackPaths) {
    if (existsSync(p)) {
      return require(p);
    }
  }

  throw new Error(
    `No prebuilt binary found for ${platform}-${arch}. ` +
    `Please build from source: npm run build`
  );
}

module.exports = loadNative();
```

## 4. CI/CD Flow (GitHub Actions)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CI/CD Pipeline                                       │
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   Trigger   │───▶│  Build &    │───▶│   Test      │───▶│   Publish   │  │
│  │  (tag push) │    │  Test Matrix│    │  (Node.js)  │    │             │  │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘  │
│                            │                                               │
│                            ▼                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Build Matrix (GitHub Actions)                                      │   │
│  │                                                                     │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │   │
│  │  │  macOS x64   │  │  macOS arm64 │  │  Linux x64   │            │   │
│  │  │  (build)     │  │  (build)     │  │  (build)     │            │   │
│  │  │  ├─ cargo    │  │  ├─ cargo    │  │  ├─ cargo    │            │   │
│  │  │  │  build     │  │  │  build     │  │  │  build     │            │   │
│  │  │  │  --release │  │  │  --release │  │  │  --release │            │   │
│  │  │  ├─ napi      │  │  ├─ napi      │  │  ├─ napi      │            │   │
│  │  │  │  build     │  │  │  build     │  │  │  build     │            │   │
│  │  │  └─ upload    │  │  │  └─ upload  │  │  │  └─ upload  │            │   │
│  │  │     artifact │  │  │     artifact │  │  │     artifact │            │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘            │   │
│  │                                                                     │   │
│  │  ┌──────────────┐  ┌──────────────┐                               │   │
│  │  │  Linux arm64 │  │  Windows x64 │                               │   │
│  │  │  (cross)     │  │  (build)     │                               │   │
│  │  │  ├─ cross    │  │  ├─ cargo    │                               │   │
│  │  │  │  build     │  │  │  build     │                               │   │
│  │  │  ├─ napi      │  │  │  --release │                               │   │
│  │  │  │  build     │  │  ├─ napi      │                               │   │
│  │  │  └─ upload    │  │  │  build     │                               │   │
│  │  │     artifact │  │  └─ upload    │                               │   │
│  │  └──────────────┘  │     artifact │                               │   │
│  │                    └──────────────┘                               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Publish Job (after all builds pass)                              │   │
│  │                                                                     │   │
│  │  1. Download all prebuild artifacts                                 │   │
│  │  2. Copy to npm/prebuilds/<platform>-<arch>/                        │   │
│  │  3. npm publish (with prebuilds included)                           │   │
│  │  4. cargo publish (for Rust crate)                                  │   │
│  │  5. GitHub Release with binaries                                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.1 GitHub Actions Workflow

File: `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  # ─── Build native binaries for each platform ─────────────────────
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-apple-darwin
            os: macos-latest
            arch: x64
          - target: aarch64-apple-darwin
            os: macos-latest
            arch: arm64
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            arch: x64
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            arch: arm64
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            arch: x64

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install napi-rs CLI
        run: npm install -g @napi-rs/cli

      - name: Build Rust (native)
        if: matrix.target != 'aarch64-unknown-linux-gnu'
        run: cargo build --release --target ${{ matrix.target }}

      - name: Build Rust (cross)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: |
          cargo install cross
          cross build --release --target ${{ matrix.target }}

      - name: Build napi module
        run: napi build --platform --release --target ${{ matrix.target }}
        working-directory: ./crates/napi

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: prebuild-${{ matrix.target }}
          path: crates/napi/*.node

  # ─── Test on each platform ────────────────────────────────────────
  test:
    name: Test ${{ matrix.os }}
    needs: build
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - uses: dtolnay/rust-action@stable
      - name: Download prebuilds
        uses: actions/download-artifact@v4
        with:
          path: npm/prebuilds
          pattern: prebuild-*
          merge-multiple: true
      - name: Run tests
        run: |
          cd npm
          npm install
          npm test

  # ─── Publish npm + cargo + GitHub Release ───────────────────────
  publish:
    name: Publish
    needs: [build, test]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all prebuilds
        uses: actions/download-artifact@v4
        with:
          path: npm/prebuilds
          pattern: prebuild-*
          merge-multiple: true

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Publish npm
        run: |
          cd npm
          npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

      - name: Publish cargo
        run: cargo publish --token ${{ secrets.CARGO_TOKEN }}

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: npm/prebuilds/**/*.node
          generate_release_notes: true
```

## 5. Workspace Structure (Cargo + npm)

```
od-cli-scanner/                     # Git repo root
├── Cargo.toml                      # Workspace manifest
├── crates/
│   ├── od-cli-scanner/             # Core Rust library (existing)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── core/
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   ├── detector.rs
│   │       │   ├── probe.rs
│   │       │   └── executables.rs
│   │       └── cli/
│   │           └── mod.rs
│   │
│   ├── napi/                       # napi-rs binding crate (NEW)
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── od-scan/                    # CLI binary crate (existing, renamed)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── npm/                            # npm package (NEW)
│   ├── package.json
│   ├── index.js
│   ├── index.d.ts
│   ├── native.js
│   ├── native.d.ts
│   ├── src/
│   │   └── index.ts
│   ├── test.js
│   └── README.md
│
├── .github/
│   └── workflows/
│       └── release.yml
│
└── README.md
```

### 5.1 Root Cargo.toml (Workspace)

```toml
[workspace]
members = [
    "crates/od-cli-scanner",
    "crates/napi",
    "crates/od-scan",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
authors = ["Open Design Contributors"]
edition = "2021"
license = "MIT"
repository = "https://github.com/nexu-io/od-cli-scanner"
```

### 5.2 crates/napi/Cargo.toml

```toml
[package]
name = "od-cli-scanner-napi"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Node.js bindings for od-cli-scanner via napi-rs"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "2", features = ["napi8", "tokio_rt"] }
napi-derive = "2"
od-cli-scanner = { path = "../od-cli-scanner" }

[build-dependencies]
napi-build = "2"
```

### 5.3 crates/napi/build.rs

```rust
extern crate napi_build;

fn main() {
    napi_build::setup();
}
```

## 6. Integration with Open Design Project

### 6.1 Open Design as Consumer

```typescript
// In the Open Design project:
import { scanAgents, type DetectionResult } from 'od-cli-scanner';

async function detectAvailableAgents(): Promise<DetectionResult> {
  const result = await scanAgents({
    availableOnly: true,
  });
  return result;
}
```

### 6.2 Open Design as Contributor

The Open Design project can:
1. **Add agent definitions** — PRs to `crates/od-cli-scanner/src/core/defs/` or `npm/src/index.ts`
2. **Extend probe logic** — Add new probe functions in Rust
3. **Consume via npm** — `npm install od-cli-scanner`
4. **Consume via Cargo** — `cargo add od-cli-scanner`

### 6.3 Version Sync Strategy

| Component | Version Source | Sync Method |
|-----------|---------------|-------------|
| Rust crate | `Cargo.toml` | Manual bump |
| npm package | `package.json` | Manual bump (same semver) |
| CLI binary | `Cargo.toml` | Same as crate |
| Git tag | `v{semver}` | CI trigger |

All three artifacts share the same version number. Release is atomic:
- One git tag → triggers all three publishes
- If any publish fails, the release is rolled back

## 7. Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Binding framework | napi-rs | Best-in-class, async support, TypeScript gen |
| Async runtime | tokio (already used) | Native async/await in Rust |
| Prebuild strategy | GitHub Actions matrix | Covers 99% of users without compilation |
| Fallback build | Source build via `napi-rs` | Power users / unsupported platforms |
| Package structure | Workspace with `crates/` + `npm/` | Clean separation, dual publish |
| Type generation | napi-rs auto-generates | `native.d.ts` from Rust macros |
| Node version support | >= 16 | N-API v8 compatibility |

## 8. Quick Start (for developers)

```bash
# 1. Clone and setup workspace
git clone https://github.com/nexu-io/od-cli-scanner.git
cd od-cli-scanner

# 2. Build Rust core
cargo build --release

# 3. Build napi bindings
cd crates/napi
napi build --platform

# 4. Test npm package
cd ../../npm
npm install
npm link ../crates/napi  # or copy .node file
node test.js

# 5. Publish (CI does this automatically on git tag)
# cargo publish
# npm publish
```
