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
    pub duration_ms: u32,
}

#[napi(object)]
pub struct JsAgentDef {
    pub id: String,
    pub name: String,
    pub bin: String,
    pub fallback_bins: Vec<String>,
    pub version_args: Vec<String>,
    pub version_probe_timeout_ms: u32,
    pub fallback_models: Vec<JsModelOption>,
    pub stream_format: String,
    pub install_url: Option<String>,
    pub docs_url: Option<String>,
    pub bin_env_key: Option<String>,
    pub auth_probe_args: Option<Vec<String>>,
    pub auth_probe_timeout_ms: Option<u32>,
    pub list_models_args: Option<Vec<String>>,
    pub list_models_timeout_ms: Option<u32>,
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
            duration_ms: r.duration_ms as u32,
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
            version_probe_timeout_ms: d.version_probe_timeout_ms as u32,
            fallback_models: d.fallback_models.into_iter().map(Into::into).collect(),
            stream_format: d.stream_format,
            install_url: d.install_url,
            docs_url: d.docs_url,
            bin_env_key: d.bin_env_key,
            auth_probe_args: d.auth_probe_args,
            auth_probe_timeout_ms: d.auth_probe_timeout_ms.map(|v| v as u32),
            list_models_args: d.list_models_args,
            list_models_timeout_ms: d.list_models_timeout_ms.map(|v| v as u32),
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
        version_probe_timeout_ms: d.version_probe_timeout_ms as u64,
        fallback_models: d.fallback_models.into_iter().map(|m| ModelOption {
            id: m.id,
            label: m.label,
        }).collect(),
        stream_format: d.stream_format,
        install_url: d.install_url,
        docs_url: d.docs_url,
        bin_env_key: d.bin_env_key,
        auth_probe_args: d.auth_probe_args,
        auth_probe_timeout_ms: d.auth_probe_timeout_ms.map(|v| v as u64),
        list_models_args: d.list_models_args,
        list_models_timeout_ms: d.list_models_timeout_ms.map(|v| v as u64),
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
    timeout_ms: Option<u32>,
) -> Result<Option<String>> {
    use od_cli_scanner::core::probe::probe_version;
    use std::path::PathBuf;

    let path = PathBuf::from(&bin);
    let timeout = timeout_ms.unwrap_or(3000) as u64;
    let result = probe_version(&path, &version_args, timeout).await;

    match result {
        Ok(v) => Ok(v),
        Err(_) => Ok(None),
    }
}
