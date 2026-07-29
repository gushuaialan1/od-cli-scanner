//! Integration tests for agent detection using mock CLI fixtures.
//!
//! These tests rewrite the process PATH to point at a temp dir containing
//! mock shell-script CLIs (see tests/fixtures/bin), then run the real
//! detection pipeline (resolve_executable + probe_version + detect_agents)
//! against them. All tests that mutate PATH are serialized via `serial_test`.

use od_cli_scanner::core::registry::AgentRegistry;
use od_cli_scanner::core::types::AgentEnvConfig;
use od_cli_scanner::detect_agents;
use serde::Deserialize;
use serial_test::serial;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Guards environment mutations: restores PATH and any cleared *_BIN keys on Drop.
struct EnvGuard {
    saved_path: Option<String>,
    saved_bin_vars: Vec<(String, Option<String>)>,
}

impl EnvGuard {
    /// Save current PATH and clear known agent *_BIN override vars so host
    /// machine configuration cannot leak into the fake-PATH scenarios.
    fn new() -> Self {
        let saved_path = env::var("PATH").ok();
        let bin_keys = ["CLAUDE_BIN", "CODEX_BIN", "KIMI_BIN", "GEMINI_BIN"];
        let saved_bin_vars: Vec<(String, Option<String>)> = bin_keys
            .iter()
            .map(|k| {
                let v = env::var(k).ok();
                env::remove_var(k);
                (k.to_string(), v)
            })
            .collect();
        Self {
            saved_path,
            saved_bin_vars,
        }
    }

    fn set_path(&self, dir: &Path) {
        env::set_var("PATH", dir);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.saved_path {
            Some(p) => env::set_var("PATH", p),
            None => env::remove_var("PATH"),
        }
        for (key, val) in &self.saved_bin_vars {
            match val {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    agents: Vec<ManifestAgent>,
}

#[derive(Debug, Deserialize)]
struct ManifestAgent {
    id: String,
    bin: String,
    version_output: String,
    expected_version: String,
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_manifest() -> Manifest {
    let raw = fs::read_to_string(fixtures_dir().join("manifest.json")).unwrap();
    serde_json::from_str(&raw).unwrap()
}

/// Create a temp dir populated with executable copies of the given fixture
/// binaries, and return (tempdir, EnvGuard) with PATH already rewritten to
/// contain only that temp dir. Both must be kept alive for the test.
///
/// Unix-only: the fixture binaries are POSIX shell scripts, so executing
/// them only works on unix hosts.
#[cfg(unix)]
fn with_fake_path(bins: &[&str]) -> (tempfile::TempDir, EnvGuard) {
    let tmp = tempfile::tempdir().unwrap();
    let src_dir = fixtures_dir().join("bin");
    for bin in bins {
        let src = src_dir.join(bin);
        let dst = tmp.path().join(bin);
        fs::copy(&src, &dst).unwrap();
        #[cfg(unix)]
        fs::set_permissions(&dst, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let guard = EnvGuard::new();
    guard.set_path(tmp.path());
    (tmp, guard)
}

/// Sanity check: each mock fixture responds to `--version` with the exact
/// output declared in the manifest, and exits 1 on unknown args.
///
/// Unix-only: executing the POSIX shell-script fixtures requires a shell.
#[cfg(unix)]
#[test]
#[serial]
fn fixtures_match_manifest() {
    let manifest = load_manifest();
    for agent in &manifest.agents {
        let bin = fixtures_dir().join("bin").join(&agent.bin);
        assert!(bin.exists(), "missing fixture for {}", agent.id);

        let out = std::process::Command::new(&bin)
            .arg("--version")
            .output()
            .unwrap();
        assert!(out.status.success(), "{} --version failed", agent.id);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            stdout.trim(),
            agent.version_output,
            "{} output drifted",
            agent.id
        );

        let bad = std::process::Command::new(&bin)
            .arg("--bogus-flag")
            .output()
            .unwrap();
        assert!(
            !bad.status.success(),
            "{} should exit 1 on bad args",
            agent.id
        );
    }
}

/// Scenario 1: PATH contains only the 4 mock CLIs → all 4 agents are
/// detected as available with versions matching the manifest.
///
/// Unix-only: detection executes the POSIX shell-script fixtures.
#[cfg(unix)]
#[tokio::test]
#[serial]
async fn detects_all_mock_agents_with_expected_versions() {
    let manifest = load_manifest();
    let bins: Vec<&str> = manifest.agents.iter().map(|a| a.bin.as_str()).collect();
    let (_tmp, _guard) = with_fake_path(&bins);

    let registry = AgentRegistry::new();
    let defs: Vec<_> = registry
        .list()
        .iter()
        .filter(|d| manifest.agents.iter().any(|m| m.id == d.id))
        .cloned()
        .collect();
    assert_eq!(defs.len(), 4, "registry should define all 4 fixture agents");

    let env_config: AgentEnvConfig = AgentEnvConfig::new();
    let result = detect_agents(&defs, &env_config).await;

    for expected in &manifest.agents {
        let detected = result
            .agents
            .iter()
            .find(|a| a.id == expected.id)
            .unwrap_or_else(|| panic!("agent {} not in detection result", expected.id));
        assert!(detected.available, "{} should be available", expected.id);
        assert_eq!(
            detected.version.as_deref(),
            Some(expected.expected_version.as_str()),
            "{} version mismatch",
            expected.id
        );
        assert!(
            detected.diagnostics.is_none(),
            "{} should have no diagnostics: {:?}",
            expected.id,
            detected.diagnostics
        );
    }
}

/// Declarative check (cross-platform): the manifest parses, defines exactly
/// the 4 registry agents, every fixture file exists, and each entry is
/// internally consistent. Runs on all platforms because it never executes
/// the shell-script fixtures.
#[test]
fn manifest_is_well_formed() {
    let manifest = load_manifest();
    assert_eq!(manifest.agents.len(), 4, "manifest should define 4 agents");

    let registry = AgentRegistry::new();
    for agent in &manifest.agents {
        assert!(!agent.id.is_empty(), "agent id must not be empty");
        assert!(
            registry.get(&agent.id).is_some(),
            "manifest agent {} missing from registry",
            agent.id
        );
        assert!(
            fixtures_dir().join("bin").join(&agent.bin).exists(),
            "missing fixture binary for {}",
            agent.id
        );
        assert!(
            agent.version_output.contains(&agent.expected_version),
            "{} version_output should contain expected_version",
            agent.id
        );
    }
}

/// Scenario 2: PATH points at an empty dir → agents are unavailable with a
/// `not_on_path` diagnostic.
#[tokio::test]
#[serial]
async fn missing_mocks_report_unavailable() {
    let tmp = tempfile::tempdir().unwrap(); // intentionally left empty
    let guard = EnvGuard::new();
    guard.set_path(tmp.path());

    let registry = AgentRegistry::new();
    let ids = ["claude", "codex", "gemini", "kimi"];
    let defs: Vec<_> = ids
        .iter()
        .map(|id| registry.get(id).unwrap().clone())
        .collect();

    let env_config: AgentEnvConfig = AgentEnvConfig::new();
    let result = detect_agents(&defs, &env_config).await;

    for agent in &result.agents {
        assert!(!agent.available, "{} should be unavailable", agent.id);
        assert!(
            agent.version.is_none(),
            "{} should have no version",
            agent.id
        );
        let diags = agent.diagnostics.as_ref().expect("diagnostics expected");
        assert!(
            diags.iter().any(|d| d.kind == "not_on_path"),
            "{} should report not_on_path, got {:?}",
            agent.id,
            diags.iter().map(|d| &d.kind).collect::<Vec<_>>()
        );
    }
}
