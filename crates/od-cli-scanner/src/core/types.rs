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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_option_equality() {
        let a = ModelOption {
            id: "gpt-4".to_string(),
            label: "GPT-4".to_string(),
        };
        let b = ModelOption {
            id: "gpt-4".to_string(),
            label: "GPT-4".to_string(),
        };
        let c = ModelOption {
            id: "gpt-3.5".to_string(),
            label: "GPT-3.5".to_string(),
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn auth_status_serialization() {
        assert_eq!(serde_json::to_string(&AuthStatus::Ok).unwrap(), "\"ok\"");
        assert_eq!(
            serde_json::to_string(&AuthStatus::Missing).unwrap(),
            "\"missing\""
        );
        assert_eq!(
            serde_json::to_string(&AuthStatus::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn auth_status_deserialization() {
        let ok: AuthStatus = serde_json::from_str("\"ok\"").unwrap();
        let missing: AuthStatus = serde_json::from_str("\"missing\"").unwrap();
        let unknown: AuthStatus = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(ok, AuthStatus::Ok);
        assert_eq!(missing, AuthStatus::Missing);
        assert_eq!(unknown, AuthStatus::Unknown);
    }

    #[test]
    fn models_source_serialization() {
        assert_eq!(
            serde_json::to_string(&ModelsSource::Live).unwrap(),
            "\"live\""
        );
        assert_eq!(
            serde_json::to_string(&ModelsSource::Fallback).unwrap(),
            "\"fallback\""
        );
    }

    #[test]
    fn models_source_deserialization() {
        let live: ModelsSource = serde_json::from_str("\"live\"").unwrap();
        let fallback: ModelsSource = serde_json::from_str("\"fallback\"").unwrap();
        assert_eq!(live, ModelsSource::Live);
        assert_eq!(fallback, ModelsSource::Fallback);
    }

    #[test]
    fn agent_def_default_timeout() {
        let json = r#"{"id":"test","name":"Test","bin":"test","stream_format":"json"}"#;
        let def: AgentDef = serde_json::from_str(json).unwrap();
        assert_eq!(def.version_probe_timeout_ms, 3000);
        assert!(def.fallback_bins.is_empty());
        assert!(def.version_args.is_empty());
        assert!(def.fallback_models.is_empty());
    }

    #[test]
    fn agent_def_full_deserialization() {
        let json = r#"{
            "id": "claude",
            "name": "Claude Code",
            "bin": "claude",
            "fallback_bins": ["claude-code"],
            "version_args": ["--version"],
            "version_probe_timeout_ms": 5000,
            "fallback_models": [{"id":"sonnet","label":"Sonnet"}],
            "stream_format": "anthropic",
            "install_url": "https://example.com",
            "bin_env_key": "CLAUDE_BIN",
            "auth_probe_args": ["auth","status"]
        }"#;
        let def: AgentDef = serde_json::from_str(json).unwrap();
        assert_eq!(def.id, "claude");
        assert_eq!(def.name, "Claude Code");
        assert_eq!(def.bin, "claude");
        assert_eq!(def.fallback_bins, vec!["claude-code"]);
        assert_eq!(def.version_args, vec!["--version"]);
        assert_eq!(def.version_probe_timeout_ms, 5000);
        assert_eq!(def.fallback_models.len(), 1);
        assert_eq!(def.stream_format, "anthropic");
        assert_eq!(def.install_url, Some("https://example.com".to_string()));
        assert_eq!(def.bin_env_key, Some("CLAUDE_BIN".to_string()));
        assert_eq!(
            def.auth_probe_args,
            Some(vec!["auth".to_string(), "status".to_string()])
        );
    }

    #[test]
    fn detected_agent_serialization_roundtrip() {
        let agent = DetectedAgent {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            bin: "claude".to_string(),
            available: true,
            path: Some("/usr/bin/claude".to_string()),
            version: Some("1.0.0".to_string()),
            models: vec![ModelOption {
                id: "sonnet".to_string(),
                label: "Sonnet".to_string(),
            }],
            models_source: ModelsSource::Live,
            auth_status: Some(AuthStatus::Ok),
            auth_message: None,
            diagnostics: None,
            stream_format: Some("anthropic".to_string()),
            install_url: None,
            docs_url: None,
        };

        let json = serde_json::to_string(&agent).unwrap();
        let back: DetectedAgent = serde_json::from_str(&json).unwrap();
        assert_eq!(agent.id, back.id);
        assert_eq!(agent.available, back.available);
        assert_eq!(agent.path, back.path);
    }

    #[test]
    fn detection_result_serialization() {
        let result = DetectionResult {
            agents: vec![],
            scanned_at: "2024-01-01T00:00:00Z".to_string(),
            duration_ms: 42,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("2024-01-01T00:00:00Z"));
        assert!(json.contains("42"));
    }

    #[test]
    fn agent_diagnostic_serialization() {
        let diag = AgentDiagnostic {
            kind: "not_on_path".to_string(),
            message: "Binary not found".to_string(),
            fix_actions: Some(vec![FixAction {
                kind: "install".to_string(),
                label: Some("Install".to_string()),
            }]),
        };
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("not_on_path"));
        assert!(json.contains("Binary not found"));
    }
}
