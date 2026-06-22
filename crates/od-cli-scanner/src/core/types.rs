use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an agent (e.g., "claude", "codex", "kimi")
pub type AgentId = String;

/// A single model option available for an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelOption {
    pub id: String,
    pub label: String,
}

/// Authentication status for an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthStatus {
    Ok,
    Missing,
    Unknown,
}

/// A diagnostic message for an unavailable agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiagnostic {
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_actions: Option<Vec<FixAction>>,
}

/// A suggested fix action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixAction {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Source of model list: live from CLI or static fallback
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelsSource {
    Live,
    Fallback,
}

/// Result of detecting a single agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAgent {
    pub id: AgentId,
    pub name: String,
    pub bin: String,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub models: Vec<ModelOption>,
    pub models_source: ModelsSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_status: Option<AuthStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<AgentDiagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

/// Full detection result for all agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub agents: Vec<DetectedAgent>,
    pub scanned_at: String,
    pub duration_ms: u64,
}

/// Static definition of an agent to detect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub id: AgentId,
    pub name: String,
    pub bin: String,
    #[serde(default)]
    pub fallback_bins: Vec<String>,
    #[serde(default)]
    pub version_args: Vec<String>,
    #[serde(default = "default_timeout")]
    pub version_probe_timeout_ms: u64,
    #[serde(default)]
    pub fallback_models: Vec<ModelOption>,
    pub stream_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_env_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_probe_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_probe_timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_models_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_models_timeout_ms: Option<u64>,
}

fn default_timeout() -> u64 {
    3000
}

/// Per-agent environment overrides
pub type AgentEnvConfig = HashMap<AgentId, HashMap<String, String>>;
