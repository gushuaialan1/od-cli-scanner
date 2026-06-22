use clap::Parser;
use od_cli_scanner::core::detector::detect_agents;
use od_cli_scanner::core::types::{AgentDef, AgentEnvConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "od-scan")]
#[command(about = "Detect installed AI coding agents on your system")]
#[command(version)]
struct Cli {
    /// Custom config file (JSON with agent definitions)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,

    /// Only show available agents
    #[arg(long)]
    available_only: bool,

    /// Filter by agent ID (comma-separated)
    #[arg(long)]
    filter: Option<String>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Setup tracing
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!("Starting od-scan");

    // Load agent definitions
    let defs = if let Some(config_path) = cli.config {
        info!("Loading config from {:?}", config_path);
        let content = std::fs::read_to_string(&config_path).expect("Failed to read config file");
        serde_json::from_str::<Vec<AgentDef>>(&content).expect("Invalid config JSON")
    } else {
        // Default built-in definitions
        get_default_defs()
    };

    let env_config: AgentEnvConfig = HashMap::new();

    info!("Scanning {} agents...", defs.len());
    let result = detect_agents(&defs, &env_config).await;

    // Filter if needed
    let mut agents = result.agents;
    if cli.available_only {
        agents.retain(|a| a.available);
    }
    if let Some(filter) = cli.filter {
        let ids: Vec<String> = filter.split(',').map(|s| s.trim().to_string()).collect();
        let id_set: std::collections::HashSet<String> = ids.into_iter().collect();
        agents.retain(|a| id_set.contains(&a.id));
    }

    // Output
    match cli.format.as_str() {
        "json" => {
            let output = if cli.pretty {
                serde_json::to_string_pretty(&agents).unwrap()
            } else {
                serde_json::to_string(&agents).unwrap()
            };
            println!("{}", output);
        }
        "table" => {
            println!(
                "{:<12} {:<20} {:<10} {}",
                "ID", "Name", "Available", "Version"
            );
            println!("{}", "-".repeat(60));
            for agent in &agents {
                println!(
                    "{:<12} {:<20} {:<10} {}",
                    agent.id,
                    agent.name,
                    if agent.available { "✓" } else { "✗" },
                    agent.version.as_deref().unwrap_or("-")
                );
            }
        }
        _ => {
            warn!("Unknown format: {}", cli.format);
            println!("{}", serde_json::to_string(&agents).unwrap());
        }
    }

    info!("Scan complete. Duration: {}ms", result.duration_ms);
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
            fallback_models: vec![od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "gpt-5.5".to_string(),
                    label: "GPT 5.5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gpt-5.4".to_string(),
                    label: "GPT 5.4".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "default".to_string(),
                    label: "Default".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "kimi-k2-turbo-preview".to_string(),
                    label: "Kimi K2 Turbo".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "moonshot-v1-8k".to_string(),
                    label: "Moonshot V1 8K".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "grok-4.3".to_string(),
                    label: "Grok 4.3 (xAI)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "deepseek-v4-pro".to_string(),
                    label: "DeepSeek V4 Pro".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "gemini-3-pro-preview".to_string(),
                    label: "Gemini 3 Pro Preview".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gemini-3-flash-preview".to_string(),
                    label: "Gemini 3 Flash Preview".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gemini-2.5-pro".to_string(),
                    label: "Gemini 2.5 Pro".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gemini-2.5-flash".to_string(),
                    label: "Gemini 2.5 Flash".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "auto".to_string(),
                    label: "Auto".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "sonnet-4".to_string(),
                    label: "Sonnet 4".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "sonnet-4-thinking".to_string(),
                    label: "Sonnet 4 Thinking".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "qwen3-coder-plus".to_string(),
                    label: "Qwen3 Coder Plus".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "lite".to_string(),
                    label: "Lite".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "efficient".to_string(),
                    label: "Efficient".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "auto".to_string(),
                    label: "Auto".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "performance".to_string(),
                    label: "Performance".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "claude-sonnet-4.6".to_string(),
                    label: "Claude Sonnet 4.6".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "sonnet".to_string(),
                    label: "Sonnet".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gpt-4o".to_string(),
                    label: "GPT 4o".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "deepseek/deepseek-chat".to_string(),
                    label: "DeepSeek Chat".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "deepseek-v4-pro".to_string(),
                    label: "DeepSeek V4 Pro".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "adaptive".to_string(),
                    label: "Adaptive".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "swe".to_string(),
                    label: "SWE".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "opus".to_string(),
                    label: "Opus".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "sonnet".to_string(),
                    label: "Sonnet".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "codex".to_string(),
                    label: "Codex".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gpt".to_string(),
                    label: "GPT".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "grok-build".to_string(),
                    label: "Grok Build".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "grok-4.3".to_string(),
                    label: "Grok 4.3".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "grok-4.20-reasoning".to_string(),
                    label: "Grok 4.20 Reasoning".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "grok-4.20-non-reasoning".to_string(),
                    label: "Grok 4.20 Non-Reasoning".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
        },
        // 15. trae-cli
        AgentDef {
            id: "trae-cli".to_string(),
            name: "Trae CLI".to_string(),
            bin: "traecli".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "anthropic/claude-sonnet-4-5".to_string(),
                    label: "Claude Sonnet 4.5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "openai/gpt-5".to_string(),
                    label: "GPT 5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "smart".to_string(),
                    label: "Smart".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "deep".to_string(),
                    label: "Deep".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "anthropic/claude-sonnet-4-5".to_string(),
                    label: "Claude Sonnet 4.5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "anthropic/claude-opus-4-5".to_string(),
                    label: "Claude Opus 4.5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "openai/gpt-5".to_string(),
                    label: "GPT 5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "openai/o4-mini".to_string(),
                    label: "O4 Mini".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "google/gemini-2.5-pro".to_string(),
                    label: "Gemini 2.5 Pro".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
        },
        // 19. kiro
        AgentDef {
            id: "kiro".to_string(),
            name: "Kiro".to_string(),
            bin: "kiro-cli".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![od_cli_scanner::core::types::ModelOption {
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
        },
        // 20. kilo
        AgentDef {
            id: "kilo".to_string(),
            name: "Kilo".to_string(),
            bin: "kilo".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![od_cli_scanner::core::types::ModelOption {
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
        },
        // 21. vibe
        AgentDef {
            id: "vibe".to_string(),
            name: "Vibe".to_string(),
            bin: "vibe-acp".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "Gemini 3.1 Pro (High)".to_string(),
                    label: "Gemini 3.1 Pro (High)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "Gemini 3.1 Pro (Low)".to_string(),
                    label: "Gemini 3.1 Pro (Low)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "Gemini 3.5 Flash (High)".to_string(),
                    label: "Gemini 3.5 Flash (High)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "Gemini 3.5 Flash (Medium)".to_string(),
                    label: "Gemini 3.5 Flash (Medium)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "Gemini 3.5 Flash (Low)".to_string(),
                    label: "Gemini 3.5 Flash (Low)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "Claude Sonnet 4.6 (Thinking)".to_string(),
                    label: "Claude Sonnet 4.6 (Thinking)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "Claude Opus 4.6 (Thinking)".to_string(),
                    label: "Claude Opus 4.6 (Thinking)".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
                od_cli_scanner::core::types::ModelOption {
                    id: "glm-5.1-ioa".to_string(),
                    label: "GLM 5.1 IOA".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "glm-5v-turbo-ioa".to_string(),
                    label: "GLM 5V Turbo IOA".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "claude-opus-4.8-1m".to_string(),
                    label: "Claude Opus 4.8 1M".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "claude-opus-4.8".to_string(),
                    label: "Claude Opus 4.8".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "claude-sonnet-4.6-1m".to_string(),
                    label: "Claude Sonnet 4.6 1M".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "claude-haiku-4.5".to_string(),
                    label: "Claude Haiku 4.5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gpt-5.5".to_string(),
                    label: "GPT 5.5".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gpt-5.4".to_string(),
                    label: "GPT 5.4".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gpt-5.3-codex".to_string(),
                    label: "GPT 5.3 Codex".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "gemini-3.5-flash".to_string(),
                    label: "Gemini 3.5 Flash".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "deepseek-v4-pro-ioa".to_string(),
                    label: "DeepSeek V4 Pro IOA".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "deepseek-v4-flash-ioa".to_string(),
                    label: "DeepSeek V4 Flash IOA".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "kimi-k2.6-ioa".to_string(),
                    label: "Kimi K2.6 IOA".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
                    id: "minimax-m3-ioa".to_string(),
                    label: "Minimax M3 IOA".to_string(),
                },
                od_cli_scanner::core::types::ModelOption {
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
        },
    ]
}
