/// Options for building agent launch arguments.
#[derive(Debug, Clone, Default)]
pub struct BuildArgsOptions {
    /// The prompt text to pass to the agent.
    pub prompt: String,
    /// Optional model override (e.g. "claude-sonnet-4-20250514").
    pub model: Option<String>,
    /// Optional working directory for the agent.
    pub cwd: Option<String>,
    /// Optional extra allowed directories (for sandboxed agents).
    pub extra_allowed_dirs: Vec<String>,
    /// Additional free-form flags to append.
    pub extra_flags: Vec<String>,
}

impl BuildArgsOptions {
    /// Create options with just a prompt.
    pub fn with_prompt(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    /// Set the model override.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

use crate::core::types::AgentDef;

impl AgentDef {
    /// Build CLI arguments for launching this agent.
    ///
    /// Each agent id has its own convention; unknown agents fall back to
    /// simply passing the prompt as the only argument.
    pub fn build_args(&self, options: &BuildArgsOptions) -> Vec<String> {
        match self.id.as_str() {
            "claude" => build_claude_args(self, options),
            "codex" => build_codex_args(self, options),
            "kimi" => build_kimi_args(self, options),
            "hermes" => build_hermes_args(self, options),
            "reasonix" => build_reasonix_args(self, options),
            _ => build_fallback_args(options),
        }
    }
}

fn build_claude_args(_def: &AgentDef, options: &BuildArgsOptions) -> Vec<String> {
    let mut args = vec![
        "-p".to_string(),
        options.prompt.clone(),
        "--output-format".to_string(),
        "stream-json".to_string(),
    ];
    if let Some(model) = &options.model {
        if model != "default" {
            args.push("--model".to_string());
            args.push(model.clone());
        }
    }
    if let Some(cwd) = &options.cwd {
        args.push("--cwd".to_string());
        args.push(cwd.clone());
    }
    for dir in &options.extra_allowed_dirs {
        args.push("--allowed-dir".to_string());
        args.push(dir.clone());
    }
    args.extend(options.extra_flags.clone());
    args
}

fn build_codex_args(_def: &AgentDef, options: &BuildArgsOptions) -> Vec<String> {
    let mut args = vec![
        "--prompt".to_string(),
        options.prompt.clone(),
        "--format".to_string(),
        "json-event-stream".to_string(),
    ];
    if let Some(model) = &options.model {
        if model != "default" {
            args.push("--model".to_string());
            args.push(model.clone());
        }
    }
    if let Some(cwd) = &options.cwd {
        args.push("--dir".to_string());
        args.push(cwd.clone());
    }
    for dir in &options.extra_allowed_dirs {
        args.push("--read-dir".to_string());
        args.push(dir.clone());
    }
    args.extend(options.extra_flags.clone());
    args
}

fn build_kimi_args(_def: &AgentDef, options: &BuildArgsOptions) -> Vec<String> {
    let mut args = vec![
        "--query".to_string(),
        options.prompt.clone(),
        "--output".to_string(),
        "json".to_string(),
    ];
    if let Some(model) = &options.model {
        if model != "default" {
            args.push("--model".to_string());
            args.push(model.clone());
        }
    }
    if let Some(cwd) = &options.cwd {
        args.push("--workdir".to_string());
        args.push(cwd.clone());
    }
    args.extend(options.extra_flags.clone());
    args
}

fn build_hermes_args(_def: &AgentDef, options: &BuildArgsOptions) -> Vec<String> {
    let mut args = vec![
        "--prompt".to_string(),
        options.prompt.clone(),
        "--format".to_string(),
        "acp-json-rpc".to_string(),
    ];
    if let Some(model) = &options.model {
        if model != "default" {
            args.push("--model".to_string());
            args.push(model.clone());
        }
    }
    if let Some(cwd) = &options.cwd {
        args.push("--cwd".to_string());
        args.push(cwd.clone());
    }
    for dir in &options.extra_allowed_dirs {
        args.push("--allow-dir".to_string());
        args.push(dir.clone());
    }
    args.extend(options.extra_flags.clone());
    args
}

fn build_reasonix_args(_def: &AgentDef, options: &BuildArgsOptions) -> Vec<String> {
    let mut args = vec![
        "--prompt".to_string(),
        options.prompt.clone(),
        "--format".to_string(),
        "acp-json-rpc".to_string(),
    ];
    if let Some(model) = &options.model {
        if model != "default" {
            args.push("--model".to_string());
            args.push(model.clone());
        }
    }
    if let Some(cwd) = &options.cwd {
        args.push("--dir".to_string());
        args.push(cwd.clone());
    }
    args.extend(options.extra_flags.clone());
    args
}

fn build_fallback_args(options: &BuildArgsOptions) -> Vec<String> {
    let mut args = vec![options.prompt.clone()];
    args.extend(options.extra_flags.clone());
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AgentDef;

    fn make_agent_def(id: &str) -> AgentDef {
        AgentDef {
            id: id.to_string(),
            name: id.to_string(),
            bin: id.to_string(),
            fallback_bins: vec![],
            version_args: vec![],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![],
            stream_format: "json".to_string(),
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
        }
    }

    #[test]
    fn build_args_claude_with_model() {
        let def = make_agent_def("claude");
        let opts = BuildArgsOptions::with_prompt("hello world").with_model("sonnet-4");
        let args = def.build_args(&opts);
        assert_eq!(args[0], "-p");
        assert_eq!(args[1], "hello world");
        assert_eq!(args[2], "--output-format");
        assert_eq!(args[3], "stream-json");
        assert_eq!(args[4], "--model");
        assert_eq!(args[5], "sonnet-4");
    }

    #[test]
    fn build_args_claude_default_model_skipped() {
        let def = make_agent_def("claude");
        let opts = BuildArgsOptions::with_prompt("hello world").with_model("default");
        let args = def.build_args(&opts);
        assert!(!args.contains(&"--model".to_string()));
    }

    #[test]
    fn build_args_codex_basic() {
        let def = make_agent_def("codex");
        let opts = BuildArgsOptions::with_prompt("write tests");
        let args = def.build_args(&opts);
        assert_eq!(args[0], "--prompt");
        assert_eq!(args[1], "write tests");
        assert_eq!(args[2], "--format");
        assert_eq!(args[3], "json-event-stream");
    }

    #[test]
    fn build_args_codex_with_cwd_and_dirs() {
        let def = make_agent_def("codex");
        let opts = BuildArgsOptions {
            prompt: "refactor".to_string(),
            model: Some("gpt-5.5".to_string()),
            cwd: Some("/src".to_string()),
            extra_allowed_dirs: vec!["/lib".to_string()],
            extra_flags: vec!["--verbose".to_string()],
        };
        let args = def.build_args(&opts);
        assert!(args.contains(&"--prompt".to_string()));
        assert!(args.contains(&"refactor".to_string()));
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"gpt-5.5".to_string()));
        assert!(args.contains(&"--dir".to_string()));
        assert!(args.contains(&"/src".to_string()));
        assert!(args.contains(&"--read-dir".to_string()));
        assert!(args.contains(&"/lib".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
    }

    #[test]
    fn build_args_kimi_with_prompt() {
        let def = make_agent_def("kimi");
        let opts = BuildArgsOptions::with_prompt("explain this code");
        let args = def.build_args(&opts);
        assert_eq!(args[0], "--query");
        assert_eq!(args[1], "explain this code");
        assert_eq!(args[2], "--output");
        assert_eq!(args[3], "json");
    }

    #[test]
    fn build_args_kimi_with_model_and_cwd() {
        let def = make_agent_def("kimi");
        let opts = BuildArgsOptions {
            prompt: "summarize".to_string(),
            model: Some("kimi-k2-turbo-preview".to_string()),
            cwd: Some("/project".to_string()),
            extra_allowed_dirs: vec![],
            extra_flags: vec![],
        };
        let args = def.build_args(&opts);
        assert!(args.contains(&"--query".to_string()));
        assert!(args.contains(&"summarize".to_string()));
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"kimi-k2-turbo-preview".to_string()));
        assert!(args.contains(&"--workdir".to_string()));
        assert!(args.contains(&"/project".to_string()));
    }

    #[test]
    fn build_args_hermes_basic() {
        let def = make_agent_def("hermes");
        let opts = BuildArgsOptions::with_prompt("run task");
        let args = def.build_args(&opts);
        assert_eq!(args[0], "--prompt");
        assert_eq!(args[1], "run task");
        assert_eq!(args[2], "--format");
        assert_eq!(args[3], "acp-json-rpc");
    }

    #[test]
    fn build_args_reasonix_basic() {
        let def = make_agent_def("reasonix");
        let opts = BuildArgsOptions::with_prompt("reason about this");
        let args = def.build_args(&opts);
        assert_eq!(args[0], "--prompt");
        assert_eq!(args[1], "reason about this");
        assert_eq!(args[2], "--format");
        assert_eq!(args[3], "acp-json-rpc");
    }

    #[test]
    fn build_args_fallback_unknown_agent() {
        let def = make_agent_def("unknown-agent");
        let opts = BuildArgsOptions::with_prompt("do something");
        let args = def.build_args(&opts);
        assert_eq!(args, vec!["do something"]);
    }

    #[test]
    fn build_args_fallback_with_extra_flags() {
        let def = make_agent_def("unknown-agent");
        let opts = BuildArgsOptions {
            prompt: "do something".to_string(),
            model: None,
            cwd: None,
            extra_allowed_dirs: vec![],
            extra_flags: vec!["--flag1".to_string(), "--flag2".to_string()],
        };
        let args = def.build_args(&opts);
        assert_eq!(args, vec!["do something", "--flag1", "--flag2"]);
    }
}
