use super::types::*;
use super::version_parser::parse_version;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Probe version by running `<bin> --version` and parsing the output.
/// Returns `Ok(Some(version))` on success, `Ok(None)` if the command ran but
/// produced no parseable version, and `Err(...)` for invocation failures.
pub async fn probe_version(
    bin_path: &std::path::Path,
    args: &[String],
    timeout_ms: u64,
) -> Result<Option<String>, ProbeError> {
    let result = timeout(
        Duration::from_millis(timeout_ms),
        Command::new(bin_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let raw = stdout.trim().lines().next().unwrap_or("");
                Ok(parse_version(raw))
            } else {
                // Non-zero exit but binary ran — still invocable
                Ok(None)
            }
        }
        Ok(Err(e)) => {
            use std::io::ErrorKind;
            match e.kind() {
                ErrorKind::NotFound => {
                    Err(ProbeError::NotInvocable(NotInvocableCause::MissingTarget))
                }
                ErrorKind::PermissionDenied => {
                    Err(ProbeError::NotInvocable(NotInvocableCause::NotExecutable))
                }
                _ => {
                    // Other spawn error — binary exists but failed
                    Ok(None)
                }
            }
        }
        Err(_) => Err(ProbeError::Timeout),
    }
}

/// Probe auth status by running the agent's auth probe command.
/// Returns `Some(AuthStatus)` if the command executed, `None` on timeout or spawn failure.
/// Parses output to determine: Ok, Expired, Missing, or Unknown.
pub async fn probe_auth(
    bin_path: &std::path::Path,
    args: &[String],
    timeout_ms: u64,
) -> Option<AuthStatus> {
    let result = timeout(
        Duration::from_millis(timeout_ms),
        Command::new(bin_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr).to_lowercase();

            // Check for expired auth first (more specific than missing)
            if combined.contains("expired")
                || combined.contains("token expired")
                || combined.contains("session expired")
                || combined.contains("credentials expired")
                || combined.contains("authentication expired")
                || combined.contains("login expired")
                || combined.contains("refresh token")
                || combined.contains("re-authenticate")
            {
                Some(AuthStatus::Expired)
            } else if combined.contains("not authenticated")
                || combined.contains("login required")
                || combined.contains("no valid credentials")
                || combined.contains("unauthorized")
                || combined.contains("not logged in")
                || combined.contains("authentication required")
                || combined.contains("missing api key")
                || combined.contains("no api key")
                || combined.contains("api key not found")
            {
                Some(AuthStatus::Missing)
            } else if combined.contains("\"loggedin\": false")
                || combined.contains("\"loggedin\":false")
            {
                // Claude Code JSON response: {"loggedIn": false}
                Some(AuthStatus::Missing)
            } else if output.status.success() {
                // Success exit code with no negative auth indicators = authenticated
                // Covers: "Logged in using ChatGPT" (codex), "authenticated" (generic)
                Some(AuthStatus::Ok)
            } else {
                Some(AuthStatus::Unknown)
            }
        }
        _ => None,
    }
}

/// Probe model list by running the agent's list-models command.
/// Returns live models if the command succeeds and output is parseable,
/// otherwise falls back to the provided static list.
pub async fn probe_models(
    bin_path: &std::path::Path,
    args: Option<&[String]>,
    timeout_ms: u64,
    fallback: &[ModelOption],
) -> (Vec<ModelOption>, ModelsSource) {
    let Some(args) = args else {
        return (fallback.to_vec(), ModelsSource::Fallback);
    };

    let result = timeout(
        Duration::from_millis(timeout_ms),
        Command::new(bin_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await;

    let output = match result {
        Ok(Ok(out)) if out.status.success() => out,
        _ => return (fallback.to_vec(), ModelsSource::Fallback),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    match parse_models(&stdout) {
        Some(models) if !models.is_empty() => (models, ModelsSource::Live),
        _ => (fallback.to_vec(), ModelsSource::Fallback),
    }
}

/// Parse model list from command output.
/// Supports JSON array, JSON object with "models" key, and plain text.
fn parse_models(output: &str) -> Option<Vec<ModelOption>> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try JSON array first: [{"id":"gpt-4","label":"GPT-4"}]
    if trimmed.starts_with('[') {
        if let Ok(models) = serde_json::from_str::<Vec<ModelOption>>(trimmed) {
            return Some(models);
        }
    }

    // Try JSON object with "models" key: {"models":[{"id":"..."}]}
    if trimmed.starts_with('{') {
        if let Ok(wrapper) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(arr) = wrapper.get("models").and_then(|v| v.as_array()) {
                let models: Vec<ModelOption> = arr
                    .iter()
                    .filter_map(|v| {
                        let id = v.get("id")?.as_str()?;
                        let label = v.get("label").and_then(|l| l.as_str()).unwrap_or(id);
                        Some(ModelOption {
                            id: id.to_string(),
                            label: label.to_string(),
                        })
                    })
                    .collect();
                if !models.is_empty() {
                    return Some(models);
                }
            }
        }
    }

    // Plain text: each non-empty line is a model
    let models: Vec<ModelOption> = trimmed
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            // Split on whitespace/tabs to get id and optional label
            let mut parts = line.split_whitespace();
            let id = parts.next().unwrap_or(line).to_string();
            let label = parts.collect::<Vec<_>>().join(" ");
            let label = if label.is_empty() {
                id.clone()
            } else {
                label
            };
            ModelOption { id, label }
        })
        .collect();

    if !models.is_empty() {
        Some(models)
    } else {
        None
    }
}

/// Known capability flags to detect from --help output.
const KNOWN_FLAGS: &[(&str, &str)] = &[
    ("--model", "model"),
    ("-m", "model"),
    ("--prompt", "prompt"),
    ("-p", "prompt"),
    ("--file", "file"),
    ("-f", "file"),
    ("--stream", "stream"),
    ("--interactive", "interactive"),
    ("-i", "interactive"),
    ("--batch", "batch"),
    ("-b", "batch"),
    ("--config", "config"),
    ("-c", "config"),
    ("--verbose", "verbose"),
    ("-V", "verbose"),
    ("--quiet", "quiet"),
    ("-q", "quiet"),
    ("--output", "output"),
    ("-o", "output"),
    ("--directory", "directory"),
    ("-d", "directory"),
    ("--project", "project"),
    ("--workspace", "workspace"),
    ("-w", "workspace"),
    ("--auth", "auth"),
    ("--debug", "debug"),
    ("--dry-run", "dry-run"),
    ("--json", "json"),
    ("--markdown", "markdown"),
];

/// Probe capabilities by running `<bin> --help` (or configured `help_args`) and
/// parsing the output for known capability flags.
/// Returns a deduplicated Vec of detected capability flag names (without `--` prefix).
/// On timeout or spawn failure, returns an empty Vec.
pub async fn probe_capabilities(
    bin_path: &std::path::Path,
    help_args: Option<&[String]>,
    timeout_ms: u64,
    expected_caps: &[String],
) -> Vec<String> {
    let args: Vec<String> = help_args
        .filter(|a| !a.is_empty())
        .map(|a| a.to_vec())
        .unwrap_or_else(|| vec!["--help".to_string()]);

    let result = timeout(
        Duration::from_millis(timeout_ms),
        Command::new(bin_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await;

    let output_text = match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("{}{}", stdout, stderr)
        }
        _ => return vec![],
    };

    let mut detected = std::collections::HashSet::new();

    // Check known flags in the help output
    for (flag, cap_name) in KNOWN_FLAGS {
        if output_text.contains(flag) {
            detected.insert(cap_name.to_string());
        }
    }

    // Also check expected capabilities from AgentDef
    for cap in expected_caps {
        let flag = format!("--{}", cap);
        if output_text.contains(&flag) {
            detected.insert(cap.clone());
        }
    }

    let mut result: Vec<String> = detected.into_iter().collect();
    result.sort();
    result
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("probe timed out")]
    Timeout,
    #[error("not invocable: {0:?}")]
    NotInvocable(NotInvocableCause),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotInvocableCause {
    MissingTarget,
    NotExecutable,
}
