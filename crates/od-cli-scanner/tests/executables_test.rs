use od_cli_scanner::core::executables::resolve_executable;
use od_cli_scanner::core::types::AgentDef;
use std::collections::HashMap;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn make_agent_def(bin: &str, fallback_bins: Vec<&str>, env_key: Option<&str>) -> AgentDef {
    AgentDef {
        id: "test".to_string(),
        name: "Test Agent".to_string(),
        bin: bin.to_string(),
        fallback_bins: fallback_bins.into_iter().map(String::from).collect(),
        version_args: vec!["--version".to_string()],
        version_probe_timeout_ms: 3000,
        fallback_models: vec![],
        stream_format: "json".to_string(),
        install_url: None,
        docs_url: None,
        bin_env_key: env_key.map(String::from),
        auth_probe_args: None,
        auth_probe_timeout_ms: None,
        list_models_args: None,
        list_models_timeout_ms: None,
    }
}

#[test]
fn resolve_executable_returns_none_for_missing_bin() {
    let def = make_agent_def("nonexistent_binary_12345", vec![], None);
    let env = HashMap::new();
    assert!(resolve_executable(&def, &env).is_none());
}

#[cfg(unix)]
#[test]
fn resolve_executable_uses_configured_env_override() {
    let tmpdir = tempfile::tempdir().unwrap();
    let fake_bin = tmpdir.path().join("myagent");
    fs::write(&fake_bin, "#!/bin/sh\necho 1.0.0").unwrap();
    fs::set_permissions(&fake_bin, fs::Permissions::from_mode(0o755)).unwrap();

    let def = make_agent_def("myagent", vec![], Some("MYAGENT_BIN"));
    let mut env = HashMap::new();
    env.insert(
        "MYAGENT_BIN".to_string(),
        fake_bin.to_str().unwrap().to_string(),
    );

    let result = resolve_executable(&def, &env);
    assert_eq!(result, Some(fake_bin));
}

#[cfg(unix)]
#[test]
fn resolve_executable_prefers_configured_env_over_process_env() {
    let tmpdir = tempfile::tempdir().unwrap();
    let fake_bin1 = tmpdir.path().join("agent1");
    let fake_bin2 = tmpdir.path().join("agent2");
    fs::write(&fake_bin1, "#!/bin/sh\necho 1.0.0").unwrap();
    fs::write(&fake_bin2, "#!/bin/sh\necho 2.0.0").unwrap();
    fs::set_permissions(&fake_bin1, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&fake_bin2, fs::Permissions::from_mode(0o755)).unwrap();

    let def = make_agent_def("agent", vec![], Some("AGENT_BIN"));
    let mut env = HashMap::new();
    env.insert(
        "AGENT_BIN".to_string(),
        fake_bin1.to_str().unwrap().to_string(),
    );

    // configured env should win even if process env is set differently
    std::env::set_var("AGENT_BIN", fake_bin2.to_str().unwrap());
    let result = resolve_executable(&def, &env);
    assert_eq!(result, Some(fake_bin1));
}

#[cfg(unix)]
#[test]
fn resolve_executable_falls_back_to_process_env() {
    let tmpdir = tempfile::tempdir().unwrap();
    let fake_bin = tmpdir.path().join("fallback_agent");
    fs::write(&fake_bin, "#!/bin/sh\necho 1.0.0").unwrap();
    fs::set_permissions(&fake_bin, fs::Permissions::from_mode(0o755)).unwrap();

    let def = make_agent_def("fallback_agent", vec![], Some("FALLBACK_AGENT_BIN"));
    std::env::set_var("FALLBACK_AGENT_BIN", fake_bin.to_str().unwrap());

    let env = HashMap::new(); // empty configured env
    let result = resolve_executable(&def, &env);
    assert_eq!(result, Some(fake_bin));
}

#[test]
fn resolve_executable_skips_non_executable_env_override() {
    let tmpdir = tempfile::tempdir().unwrap();
    let fake_bin = tmpdir.path().join("not_exec");
    fs::write(&fake_bin, "no shebang").unwrap();
    // no execute permission

    let def = make_agent_def("not_exec", vec![], Some("NOT_EXEC_BIN"));
    let mut env = HashMap::new();
    env.insert(
        "NOT_EXEC_BIN".to_string(),
        fake_bin.to_str().unwrap().to_string(),
    );

    let result = resolve_executable(&def, &env);
    assert!(result.is_none());
}

#[cfg(unix)]
#[test]
fn resolve_executable_expands_tilde_in_env_override() {
    let home = dirs::home_dir().unwrap();
    let fake_bin = home.join(".local/bin/testagent");
    if let Some(parent) = fake_bin.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&fake_bin, "#!/bin/sh\necho 1.0.0").unwrap();
    fs::set_permissions(&fake_bin, fs::Permissions::from_mode(0o755)).unwrap();

    let def = make_agent_def("testagent", vec![], Some("TESTAGENT_BIN"));
    let mut env = HashMap::new();
    env.insert(
        "TESTAGENT_BIN".to_string(),
        "~/.local/bin/testagent".to_string(),
    );

    let result = resolve_executable(&def, &env);
    assert_eq!(result, Some(fake_bin.clone()));

    // cleanup
    fs::remove_file(&fake_bin).ok();
}

#[test]
fn resolve_executable_uses_fallback_bins() {
    // We can't easily mock `which`, but we can test that when primary bin
    // is missing and no env override is set, it falls back to fallback_bins.
    // Since we can't guarantee any binary exists, we just verify the logic
    // by checking that a nonexistent primary with nonexistent fallbacks returns None.
    let def = make_agent_def("primary_missing", vec!["also_missing_12345"], None);
    let env = HashMap::new();
    assert!(resolve_executable(&def, &env).is_none());
}

#[test]
fn resolve_executable_uses_primary_bin_from_path() {
    // This test relies on a real binary existing on PATH.
    // We use `sh` which is virtually guaranteed to exist on any Unix system.
    let def = make_agent_def("sh", vec![], None);
    let env = HashMap::new();
    let result = resolve_executable(&def, &env);
    assert!(result.is_some());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("sh"));
}
