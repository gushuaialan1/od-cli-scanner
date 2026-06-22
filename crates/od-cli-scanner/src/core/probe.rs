use super::types::*;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Probe version by running `<bin> --version`
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
                let version = stdout.trim().lines().next().map(|s| s.to_string());
                Ok(version)
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

/// Probe auth status (optional)
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

            if combined.contains("not authenticated")
                || combined.contains("login required")
                || combined.contains("no valid credentials")
            {
                Some(AuthStatus::Missing)
            } else if output.status.success() {
                Some(AuthStatus::Ok)
            } else {
                Some(AuthStatus::Unknown)
            }
        }
        _ => None,
    }
}

/// Probe model list (optional, simplified)
pub async fn probe_models(
    _bin_path: &std::path::Path,
    _args: Option<&[String]>,
    _timeout_ms: u64,
    fallback: &[ModelOption],
) -> (Vec<ModelOption>, ModelsSource) {
    // For v0.1, return fallback models
    // Future: implement live model probing per agent type
    (fallback.to_vec(), ModelsSource::Fallback)
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
