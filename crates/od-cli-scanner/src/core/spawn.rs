use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};

use super::types::AgentDef;

/// Options for launching an agent process.
#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    /// Extra arguments to append after the agent's default args.
    pub extra_args: Vec<String>,
    /// Environment variables to inject into the spawned process.
    pub env: Option<HashMap<String, String>>,
    /// Working directory for the process (defaults to system temp dir).
    pub cwd: Option<std::path::PathBuf>,
}

/// Spawn a child process for the given agent binary.
///
/// # Arguments
/// * `bin_path` — absolute or relative path to the executable
/// * `args`     — command-line arguments
/// * `env`      — optional environment variables to inject
///
/// # Returns
/// `Ok(Child)` on success, `Err(SpawnError)` if the binary cannot be started.
pub fn spawn_agent(
    bin_path: &Path,
    args: &[String],
    env: Option<HashMap<String, String>>,
) -> Result<Child, SpawnError> {
    let mut cmd = Command::new(bin_path);
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(vars) = env {
        for (k, v) in vars {
            cmd.env(k, v);
        }
    }

    let cwd = std::env::temp_dir();
    cmd.current_dir(&cwd);

    match cmd.spawn() {
        Ok(child) => Ok(child),
        Err(e) => {
            use std::io::ErrorKind;
            match e.kind() {
                ErrorKind::NotFound => Err(SpawnError::NotFound(bin_path.to_path_buf())),
                ErrorKind::PermissionDenied => Err(SpawnError::PermissionDenied(bin_path.to_path_buf())),
                _ => Err(SpawnError::Io(e)),
            }
        }
    }
}

/// High-level launcher that builds arguments from an `AgentDef` and spawns the process.
#[derive(Debug, Clone)]
pub struct AgentLauncher;

impl AgentLauncher {
    /// Create a new launcher instance.
    pub fn new() -> Self {
        Self
    }

    /// Build the argument list for an agent definition.
    ///
    /// Currently returns the `version_args` as the base command-line arguments.
    /// This can be extended to support sub-commands (e.g., `auth`, `list-models`).
    pub fn build_args(agent_def: &AgentDef, options: &LaunchOptions) -> Vec<String> {
        let mut args = agent_def.version_args.clone();
        args.extend_from_slice(&options.extra_args);
        args
    }

    /// Launch the agent process described by `agent_def`.
    ///
    /// # Errors
    /// Returns `SpawnError` if the binary is missing, not executable, or another IO error occurs.
    pub fn launch(
        &self,
        agent_def: &AgentDef,
        options: &LaunchOptions,
    ) -> Result<Child, SpawnError> {
        let bin_path = Path::new(&agent_def.bin);
        let args = Self::build_args(agent_def, options);
        let mut child = spawn_agent(bin_path, &args, options.env.clone())?;

        // Apply custom working directory if provided
        if let Some(ref cwd) = options.cwd {
            // Command::current_dir must be set before spawn; since spawn_agent already
            // spawned the process, we handle cwd by respawning with the override.
            // For simplicity, we re-spawn with the correct cwd.
            let _ = child.start_kill();
            let mut cmd = Command::new(bin_path);
            cmd.args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(cwd);
            if let Some(ref vars) = options.env {
                for (k, v) in vars {
                    cmd.env(k, v);
                }
            }
            return match cmd.spawn() {
                Ok(c) => Ok(c),
                Err(e) => {
                    use std::io::ErrorKind;
                    match e.kind() {
                        ErrorKind::NotFound => Err(SpawnError::NotFound(bin_path.to_path_buf())),
                        ErrorKind::PermissionDenied => Err(SpawnError::PermissionDenied(bin_path.to_path_buf())),
                        _ => Err(SpawnError::Io(e)),
                    }
                }
            };
        }

        Ok(child)
    }
}

impl Default for AgentLauncher {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur when spawning an agent process.
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("binary not found: {0}")]
    NotFound(std::path::PathBuf),
    #[error("permission denied for binary: {0}")]
    PermissionDenied(std::path::PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn spawn_agent_echo() -> Result<(), Box<dyn std::error::Error>> {
        // Use the system `echo` binary (available on Unix-like systems)
        let echo_path = Path::new("/bin/echo");
        if !echo_path.exists() {
            // Try alternative paths
            let alt = Path::new("/usr/bin/echo");
            if !alt.exists() {
                eprintln!("Skipping spawn_agent_echo: echo not found at /bin/echo or /usr/bin/echo");
                return Ok(());
            }
        }

        let args = vec!["hello".to_string()];
        let mut child = spawn_agent(echo_path, &args, None).expect("spawn should succeed");

        let stdout = child.stdout.take().expect("stdout should be piped");
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await.expect("read should succeed");

        let status = child.wait().await.expect("wait should succeed");
        assert!(status.success(), "echo should exit successfully");
        assert!(buf.contains("hello"), "output should contain 'hello'");
        Ok(())
    }

    #[tokio::test]
    async fn spawn_agent_with_env() -> Result<(), Box<dyn std::error::Error>> {
        // Use /bin/sh to print an environment variable
        let sh_path = Path::new("/bin/sh");
        if !sh_path.exists() {
            let alt = Path::new("/usr/bin/sh");
            if !alt.exists() {
                eprintln!("Skipping spawn_agent_with_env: sh not found");
                return Ok(());
            }
        }

        let mut env = HashMap::new();
        env.insert("OD_TEST_VAR".to_string(), "od_value_42".to_string());

        let args = vec!["-c".to_string(), "echo $OD_TEST_VAR".to_string()];
        let mut child = spawn_agent(sh_path, &args, Some(env)).expect("spawn should succeed");

        let stdout = child.stdout.take().expect("stdout should be piped");
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await.expect("read should succeed");

        let status = child.wait().await.expect("wait should succeed");
        assert!(status.success(), "shell should exit successfully");
        assert!(buf.contains("od_value_42"), "output should contain injected env var value");
        Ok(())
    }

    #[tokio::test]
    async fn spawn_agent_invalid_bin() -> Result<(), Box<dyn std::error::Error>> {
        let fake_path = Path::new("/nonexistent/binary/that/does/not/exist");
        let result = spawn_agent(fake_path, &[], None);
        assert!(result.is_err(), "spawn should fail for invalid binary");
        match result {
            Err(SpawnError::NotFound(_)) => {}
            Err(other) => panic!("expected NotFound error, got: {:?}", other),
            Ok(_) => panic!("expected error, got Ok"),
        }
        Ok(())
    }

    #[test]
    fn agent_launcher_build_args() {
        let agent_def = AgentDef {
            id: "test".to_string(),
            name: "Test Agent".to_string(),
            bin: "test-bin".to_string(),
            fallback_bins: vec![],
            version_args: vec!["--version".to_string()],
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
        };

        let options = LaunchOptions {
            extra_args: vec!["--verbose".to_string()],
            ..Default::default()
        };

        let args = AgentLauncher::build_args(&agent_def, &options);
        assert_eq!(args, vec!["--version", "--verbose"]);
    }

    #[tokio::test]
    async fn agent_launcher_launch_echo() -> Result<(), Box<dyn std::error::Error>> {
        let echo_path = if Path::new("/bin/echo").exists() {
            "/bin/echo"
        } else if Path::new("/usr/bin/echo").exists() {
            "/usr/bin/echo"
        } else {
            eprintln!("Skipping agent_launcher_launch_echo: echo not found");
            return Ok(());
        };

        let agent_def = AgentDef {
            id: "echo".to_string(),
            name: "Echo".to_string(),
            bin: echo_path.to_string(),
            fallback_bins: vec![],
            version_args: vec!["launcher-test".to_string()],
            version_probe_timeout_ms: 3000,
            fallback_models: vec![],
            stream_format: "text".to_string(),
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
        };

        let launcher = AgentLauncher::new();
        let options = LaunchOptions::default();
        let mut child = launcher.launch(&agent_def, &options).expect("launch should succeed");

        let stdout = child.stdout.take().expect("stdout should be piped");
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await.expect("read should succeed");

        let status = child.wait().await.expect("wait should succeed");
        assert!(status.success());
        assert!(buf.contains("launcher-test"));
        Ok(())
    }
}
