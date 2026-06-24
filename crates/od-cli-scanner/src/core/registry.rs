use super::types::{AgentDef, ModelOption};

/// Registry of agent definitions, providing lookup and listing capabilities.
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Vec<AgentDef>,
}

impl AgentRegistry {
    /// Create a new registry with the built-in default agent definitions.
    pub fn new() -> Self {
        Self {
            agents: get_default_defs(),
        }
    }

    /// Get an agent definition by its ID.
    pub fn get(&self, id: &str) -> Option<&AgentDef> {
        self.agents.iter().find(|a| a.id == id)
    }

    /// List all agent definitions.
    pub fn list(&self) -> &[AgentDef] {
        &self.agents
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn get_default_defs() -> Vec<AgentDef> {
    vec![
        // 1. claude
        AgentDef {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            bin: "claude".to_string(),
            fallback_bins: vec!["claude-code".to_string()],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![ModelOption {
                id: "claude-sonnet-4-20250514".to_string(),
                label: "Claude Sonnet 4".to_string(),
            }],
            stream_format: "anthropic".to_string(),
            install_url: Some("https://docs.anthropic.com/claude-code".to_string()),
            docs_url: Some("https://docs.anthropic.com/claude-code".to_string()),
            bin_env_key: Some("CLAUDE_BIN".to_string()),
            auth_probe_args: Some(vec!["auth".to_string(), "status".to_string()]),
            auth_probe_timeout_ms: Some(5000),
            list_models_args: Some(vec!["models".to_string()]),
            list_models_timeout_ms: Some(5000),
            help_args: Some(vec!["--help".to_string()]),
            help_probe_timeout_ms: Some(5000),
            capabilities: vec![
                "model".to_string(),
                "prompt".to_string(),
                "file".to_string(),
                "stream".to_string(),
                "interactive".to_string(),
                "batch".to_string(),
                "config".to_string(),
                "verbose".to_string(),
                "quiet".to_string(),
                "output".to_string(),
                "directory".to_string(),
                "project".to_string(),
                "workspace".to_string(),
                "auth".to_string(),
                "debug".to_string(),
                "dry-run".to_string(),
                "json".to_string(),
                "markdown".to_string(),
            ],
        },
        // 2. codex
        AgentDef {
            id: "codex".to_string(),
            name: "OpenAI Codex".to_string(),
            bin: "codex".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "gpt-5.5".to_string(),
                    label: "GPT 5.5".to_string(),
                },
                ModelOption {
                    id: "gpt-5.4".to_string(),
                    label: "GPT 5.4".to_string(),
                },
                ModelOption {
                    id: "gpt-5.4-mini".to_string(),
                    label: "GPT 5.4 Mini".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: Some("https://github.com/openai/codex".to_string()),
            docs_url: Some("https://github.com/openai/codex#readme".to_string()),
            bin_env_key: Some("CODEX_BIN".to_string()),
            auth_probe_args: Some(vec!["login".to_string(), "status".to_string()]),
            auth_probe_timeout_ms: Some(5000),
            list_models_args: Some(vec!["debug".to_string(), "models".to_string()]),
            list_models_timeout_ms: Some(5000),
            help_args: Some(vec!["--help".to_string()]),
            help_probe_timeout_ms: Some(5000),
            capabilities: vec![
                "model".to_string(),
                "prompt".to_string(),
                "file".to_string(),
                "stream".to_string(),
                "interactive".to_string(),
                "batch".to_string(),
                "config".to_string(),
                "verbose".to_string(),
                "quiet".to_string(),
                "output".to_string(),
                "directory".to_string(),
                "project".to_string(),
                "workspace".to_string(),
                "auth".to_string(),
                "debug".to_string(),
                "dry-run".to_string(),
                "json".to_string(),
                "markdown".to_string(),
            ],
        },
        // 3. kimi
        AgentDef {
            id: "kimi".to_string(),
            name: "Kimi Code CLI".to_string(),
            bin: "kimi".to_string(),
            fallback_bins: vec!["kimi-code".to_string()],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 10_000,
            fallback_models: vec![
                ModelOption {
                    id: "default".to_string(),
                    label: "Default".to_string(),
                },
                ModelOption {
                    id: "kimi-k2-turbo-preview".to_string(),
                    label: "Kimi K2 Turbo".to_string(),
                },
                ModelOption {
                    id: "moonshot-v1-8k".to_string(),
                    label: "Moonshot V1 8K".to_string(),
                },
                ModelOption {
                    id: "moonshot-v1-32k".to_string(),
                    label: "Moonshot V1 32K".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: Some("https://platform.moonshot.cn".to_string()),
            docs_url: None,
            bin_env_key: Some("KIMI_BIN".to_string()),
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 4. hermes
        AgentDef {
            id: "hermes".to_string(),
            name: "Hermes Agent".to_string(),
            bin: "hermes".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 10_000,
            fallback_models: vec![
                ModelOption {
                    id: "grok-4.3".to_string(),
                    label: "Grok 4.3 (xAI)".to_string(),
                },
                ModelOption {
                    id: "openai-codex:gpt-5.5".to_string(),
                    label: "GPT 5.5 (OpenAI)".to_string(),
                },
            ],
            stream_format: "acp-json-rpc".to_string(),
            install_url: Some("https://github.com/nousresearch/hermes-agent".to_string()),
            docs_url: None,
            bin_env_key: Some("HERMES_BIN".to_string()),
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 5. reasonix
        AgentDef {
            id: "reasonix".to_string(),
            name: "DeepSeek Reasonix".to_string(),
            bin: "reasonix".to_string(),
            fallback_bins: vec!["dsnix".to_string()],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "deepseek-v4-pro".to_string(),
                    label: "DeepSeek V4 Pro".to_string(),
                },
                ModelOption {
                    id: "deepseek-v4-flash".to_string(),
                    label: "DeepSeek V4 Flash".to_string(),
                },
            ],
            stream_format: "acp-json-rpc".to_string(),
            install_url: Some("https://github.com/esengine/DeepSeek-Reasonix".to_string()),
            docs_url: Some("https://esengine.github.io/DeepSeek-Reasonix/".to_string()),
            bin_env_key: Some("REASONIX_BIN".to_string()),
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 6. gemini
        AgentDef {
            id: "gemini".to_string(),
            name: "Gemini CLI".to_string(),
            bin: "gemini".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "gemini-3-pro-preview".to_string(),
                    label: "Gemini 3 Pro Preview".to_string(),
                },
                ModelOption {
                    id: "gemini-3-flash-preview".to_string(),
                    label: "Gemini 3 Flash Preview".to_string(),
                },
                ModelOption {
                    id: "gemini-2.5-pro".to_string(),
                    label: "Gemini 2.5 Pro".to_string(),
                },
                ModelOption {
                    id: "gemini-2.5-flash".to_string(),
                    label: "Gemini 2.5 Flash".to_string(),
                },
                ModelOption {
                    id: "gemini-2.5-flash-lite".to_string(),
                    label: "Gemini 2.5 Flash Lite".to_string(),
                },
            ],
            stream_format: "google".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 7. cursor-agent
        AgentDef {
            id: "cursor-agent".to_string(),
            name: "Cursor Agent".to_string(),
            bin: "cursor-agent".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "auto".to_string(),
                    label: "Auto".to_string(),
                },
                ModelOption {
                    id: "sonnet-4".to_string(),
                    label: "Sonnet 4".to_string(),
                },
                ModelOption {
                    id: "sonnet-4-thinking".to_string(),
                    label: "Sonnet 4 Thinking".to_string(),
                },
                ModelOption {
                    id: "gpt-5".to_string(),
                    label: "GPT 5".to_string(),
                },
            ],
            stream_format: "anthropic".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 8. qwen
        AgentDef {
            id: "qwen".to_string(),
            name: "Qwen CLI".to_string(),
            bin: "qwen".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "qwen3-coder-plus".to_string(),
                    label: "Qwen3 Coder Plus".to_string(),
                },
                ModelOption {
                    id: "qwen3-coder-flash".to_string(),
                    label: "Qwen3 Coder Flash".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 9. qoder
        AgentDef {
            id: "qoder".to_string(),
            name: "Qoder".to_string(),
            bin: "qodercli".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "lite".to_string(),
                    label: "Lite".to_string(),
                },
                ModelOption {
                    id: "efficient".to_string(),
                    label: "Efficient".to_string(),
                },
                ModelOption {
                    id: "auto".to_string(),
                    label: "Auto".to_string(),
                },
                ModelOption {
                    id: "performance".to_string(),
                    label: "Performance".to_string(),
                },
                ModelOption {
                    id: "ultimate".to_string(),
                    label: "Ultimate".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 10. copilot
        AgentDef {
            id: "copilot".to_string(),
            name: "GitHub Copilot CLI".to_string(),
            bin: "copilot".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "claude-sonnet-4.6".to_string(),
                    label: "Claude Sonnet 4.6".to_string(),
                },
                ModelOption {
                    id: "gpt-5.2".to_string(),
                    label: "GPT 5.2".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 11. aider
        AgentDef {
            id: "aider".to_string(),
            name: "Aider".to_string(),
            bin: "aider".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "sonnet".to_string(),
                    label: "Sonnet".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    label: "GPT 4o".to_string(),
                },
                ModelOption {
                    id: "deepseek/deepseek-chat".to_string(),
                    label: "DeepSeek Chat".to_string(),
                },
                ModelOption {
                    id: "gemini/gemini-2.0-flash".to_string(),
                    label: "Gemini 2.0 Flash".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 12. deepseek
        AgentDef {
            id: "deepseek".to_string(),
            name: "DeepSeek CLI".to_string(),
            bin: "deepseek".to_string(),
            fallback_bins: vec!["codewhale".to_string()],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "deepseek-v4-pro".to_string(),
                    label: "DeepSeek V4 Pro".to_string(),
                },
                ModelOption {
                    id: "deepseek-v4-flash".to_string(),
                    label: "DeepSeek V4 Flash".to_string(),
                },
            ],
            stream_format: "acp-json-rpc".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 13. devin
        AgentDef {
            id: "devin".to_string(),
            name: "Devin".to_string(),
            bin: "devin".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "adaptive".to_string(),
                    label: "Adaptive".to_string(),
                },
                ModelOption {
                    id: "swe".to_string(),
                    label: "SWE".to_string(),
                },
                ModelOption {
                    id: "opus".to_string(),
                    label: "Opus".to_string(),
                },
                ModelOption {
                    id: "sonnet".to_string(),
                    label: "Sonnet".to_string(),
                },
                ModelOption {
                    id: "codex".to_string(),
                    label: "Codex".to_string(),
                },
                ModelOption {
                    id: "gpt".to_string(),
                    label: "GPT".to_string(),
                },
                ModelOption {
                    id: "gemini".to_string(),
                    label: "Gemini".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 14. grok-build
        AgentDef {
            id: "grok-build".to_string(),
            name: "Grok Build".to_string(),
            bin: "grok".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "grok-build".to_string(),
                    label: "Grok Build".to_string(),
                },
                ModelOption {
                    id: "grok-4.3".to_string(),
                    label: "Grok 4.3".to_string(),
                },
                ModelOption {
                    id: "grok-4.20-reasoning".to_string(),
                    label: "Grok 4.20 Reasoning".to_string(),
                },
                ModelOption {
                    id: "grok-4.20-non-reasoning".to_string(),
                    label: "Grok 4.20 Non-Reasoning".to_string(),
                },
                ModelOption {
                    id: "grok-4.20-multi-agent".to_string(),
                    label: "Grok 4.20 Multi-Agent".to_string(),
                },
            ],
            stream_format: "xai".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 15. trae-cli
        AgentDef {
            id: "trae-cli".to_string(),
            name: "Trae CLI".to_string(),
            bin: "traecli".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![ModelOption {
                id: "default".to_string(),
                label: "Default".to_string(),
            }],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 16. opencode
        AgentDef {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            bin: "opencode-cli".to_string(),
            fallback_bins: vec!["opencode".to_string()],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "anthropic/claude-sonnet-4-5".to_string(),
                    label: "Claude Sonnet 4.5".to_string(),
                },
                ModelOption {
                    id: "openai/gpt-5".to_string(),
                    label: "GPT 5".to_string(),
                },
                ModelOption {
                    id: "google/gemini-2.5-pro".to_string(),
                    label: "Gemini 2.5 Pro".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 17. amp
        AgentDef {
            id: "amp".to_string(),
            name: "Amp".to_string(),
            bin: "amp".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "smart".to_string(),
                    label: "Smart".to_string(),
                },
                ModelOption {
                    id: "deep".to_string(),
                    label: "Deep".to_string(),
                },
                ModelOption {
                    id: "rush".to_string(),
                    label: "Rush".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 18. pi
        AgentDef {
            id: "pi".to_string(),
            name: "Pi".to_string(),
            bin: "pi".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "anthropic/claude-sonnet-4-5".to_string(),
                    label: "Claude Sonnet 4.5".to_string(),
                },
                ModelOption {
                    id: "anthropic/claude-opus-4-5".to_string(),
                    label: "Claude Opus 4.5".to_string(),
                },
                ModelOption {
                    id: "openai/gpt-5".to_string(),
                    label: "GPT 5".to_string(),
                },
                ModelOption {
                    id: "openai/o4-mini".to_string(),
                    label: "O4 Mini".to_string(),
                },
                ModelOption {
                    id: "google/gemini-2.5-pro".to_string(),
                    label: "Gemini 2.5 Pro".to_string(),
                },
                ModelOption {
                    id: "google/gemini-2.5-flash".to_string(),
                    label: "Gemini 2.5 Flash".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 19. kiro
        AgentDef {
            id: "kiro".to_string(),
            name: "Kiro".to_string(),
            bin: "kiro-cli".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![ModelOption {
                id: "default".to_string(),
                label: "Default".to_string(),
            }],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 20. kilo
        AgentDef {
            id: "kilo".to_string(),
            name: "Kilo".to_string(),
            bin: "kilo".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![ModelOption {
                id: "default".to_string(),
                label: "Default".to_string(),
            }],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 21. vibe
        AgentDef {
            id: "vibe".to_string(),
            name: "Vibe".to_string(),
            bin: "vibe-acp".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![ModelOption {
                id: "default".to_string(),
                label: "Default".to_string(),
            }],
            stream_format: "acp-json-rpc".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 22. antigravity
        AgentDef {
            id: "antigravity".to_string(),
            name: "Antigravity".to_string(),
            bin: "agy".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "Gemini 3.1 Pro (High)".to_string(),
                    label: "Gemini 3.1 Pro (High)".to_string(),
                },
                ModelOption {
                    id: "Gemini 3.1 Pro (Low)".to_string(),
                    label: "Gemini 3.1 Pro (Low)".to_string(),
                },
                ModelOption {
                    id: "Gemini 3.5 Flash (High)".to_string(),
                    label: "Gemini 3.5 Flash (High)".to_string(),
                },
                ModelOption {
                    id: "Gemini 3.5 Flash (Medium)".to_string(),
                    label: "Gemini 3.5 Flash (Medium)".to_string(),
                },
                ModelOption {
                    id: "Gemini 3.5 Flash (Low)".to_string(),
                    label: "Gemini 3.5 Flash (Low)".to_string(),
                },
                ModelOption {
                    id: "Claude Sonnet 4.6 (Thinking)".to_string(),
                    label: "Claude Sonnet 4.6 (Thinking)".to_string(),
                },
                ModelOption {
                    id: "Claude Opus 4.6 (Thinking)".to_string(),
                    label: "Claude Opus 4.6 (Thinking)".to_string(),
                },
                ModelOption {
                    id: "GPT-OSS 120B (Medium)".to_string(),
                    label: "GPT-OSS 120B (Medium)".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 23. codebuddy
        AgentDef {
            id: "codebuddy".to_string(),
            name: "CodeBuddy".to_string(),
            bin: "codebuddy".to_string(),
            fallback_bins: vec!["cbc".to_string()],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![
                ModelOption {
                    id: "glm-5.1-ioa".to_string(),
                    label: "GLM 5.1 IOA".to_string(),
                },
                ModelOption {
                    id: "glm-5v-turbo-ioa".to_string(),
                    label: "GLM 5V Turbo IOA".to_string(),
                },
                ModelOption {
                    id: "claude-opus-4.8-1m".to_string(),
                    label: "Claude Opus 4.8 1M".to_string(),
                },
                ModelOption {
                    id: "claude-opus-4.8".to_string(),
                    label: "Claude Opus 4.8".to_string(),
                },
                ModelOption {
                    id: "claude-sonnet-4.6-1m".to_string(),
                    label: "Claude Sonnet 4.6 1M".to_string(),
                },
                ModelOption {
                    id: "claude-haiku-4.5".to_string(),
                    label: "Claude Haiku 4.5".to_string(),
                },
                ModelOption {
                    id: "gpt-5.5".to_string(),
                    label: "GPT 5.5".to_string(),
                },
                ModelOption {
                    id: "gpt-5.4".to_string(),
                    label: "GPT 5.4".to_string(),
                },
                ModelOption {
                    id: "gpt-5.3-codex".to_string(),
                    label: "GPT 5.3 Codex".to_string(),
                },
                ModelOption {
                    id: "gemini-3.5-flash".to_string(),
                    label: "Gemini 3.5 Flash".to_string(),
                },
                ModelOption {
                    id: "deepseek-v4-pro-ioa".to_string(),
                    label: "DeepSeek V4 Pro IOA".to_string(),
                },
                ModelOption {
                    id: "deepseek-v4-flash-ioa".to_string(),
                    label: "DeepSeek V4 Flash IOA".to_string(),
                },
                ModelOption {
                    id: "kimi-k2.6-ioa".to_string(),
                    label: "Kimi K2.6 IOA".to_string(),
                },
                ModelOption {
                    id: "minimax-m3-ioa".to_string(),
                    label: "Minimax M3 IOA".to_string(),
                },
                ModelOption {
                    id: "minimax-m2.7-ioa".to_string(),
                    label: "Minimax M2.7 IOA".to_string(),
                },
            ],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
        // 24. amr
        AgentDef {
            id: "amr".to_string(),
            name: "AMR".to_string(),
            bin: "vela".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![],
            stream_format: "json-event-stream".to_string(),
            install_url: None,
            docs_url: None,
            bin_env_key: None,
            auth_probe_args: None,
            auth_probe_timeout_ms: None,
            list_models_args: None,
            list_models_timeout_ms: None,
            help_args: None,
            help_probe_timeout_ms: None,
            capabilities: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_get_existing_agent() {
        let registry = AgentRegistry::new();
        let agent = registry.get("claude");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().id, "claude");
    }

    #[test]
    fn registry_get_nonexistent_agent() {
        let registry = AgentRegistry::new();
        let agent = registry.get("nonexistent");
        assert!(agent.is_none());
    }

    #[test]
    fn registry_list_returns_all() {
        let registry = AgentRegistry::new();
        let agents = registry.list();
        assert_eq!(agents.len(), 24);
        assert!(agents.iter().any(|a| a.id == "claude"));
        assert!(agents.iter().any(|a| a.id == "codex"));
        assert!(agents.iter().any(|a| a.id == "amr"));
    }
}
