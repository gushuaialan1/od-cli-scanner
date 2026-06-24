use super::executables;
use super::probe;
use super::types::*;
use futures::future::join_all;
use std::time::Instant;

/// Detect all configured agents concurrently.
pub async fn detect_agents(defs: &[AgentDef], configured_env: &AgentEnvConfig) -> DetectionResult {
    let start = Instant::now();

    let futures: Vec<_> = defs
        .iter()
        .map(|def| detect_single_agent(def, configured_env))
        .collect();

    let agents = join_all(futures).await;

    DetectionResult {
        agents,
        scanned_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn detect_single_agent(def: &AgentDef, configured_env: &AgentEnvConfig) -> DetectedAgent {
    let env_for_agent = configured_env.get(&def.id).cloned().unwrap_or_default();

    // 1. Resolve executable
    let Some(bin_path) = executables::resolve_executable(def, &env_for_agent) else {
        return make_unavailable(
            def,
            vec![AgentDiagnostic {
                kind: "not_on_path".into(),
                message: format!(
                    "{} not found on PATH. Install it or set {}.",
                    def.name,
                    def.bin_env_key
                        .as_deref()
                        .unwrap_or(&format!("{}_BIN", def.id.to_uppercase()))
                ),
                fix_actions: Some(vec![
                    FixAction {
                        kind: "install".into(),
                        label: Some(format!("Install {}", def.name)),
                    },
                    FixAction {
                        kind: "setEnv".into(),
                        label: def.bin_env_key.as_ref().map(|k| format!("Set {}", k)),
                    },
                ]),
            }],
        );
    };

    // 2. Probe version
    let version_result =
        probe::probe_version(&bin_path, &def.version_args, def.version_probe_timeout_ms).await;

    let (version, available, diagnostics) = match version_result {
        Ok(v) => (v, true, vec![]),
        Err(probe::ProbeError::NotInvocable(cause)) => {
            let diag = match cause {
                probe::NotInvocableCause::NotExecutable => AgentDiagnostic {
                    kind: "not_executable".into(),
                    message: format!(
                        "{} exists but is not executable. Check permissions.",
                        bin_path.display()
                    ),
                    fix_actions: Some(vec![FixAction {
                        kind: "chmod".into(),
                        label: None,
                    }]),
                },
                probe::NotInvocableCause::MissingTarget => AgentDiagnostic {
                    kind: "missing_target".into(),
                    message: format!(
                        "{} wrapper found but target binary is missing.",
                        bin_path.display()
                    ),
                    fix_actions: Some(vec![FixAction {
                        kind: "reinstall".into(),
                        label: None,
                    }]),
                },
            };
            return make_unavailable(def, vec![diag]);
        }
        Err(probe::ProbeError::Timeout) => {
            return make_unavailable(
                def,
                vec![AgentDiagnostic {
                    kind: "timeout".into(),
                    message: "Version probe timed out".into(),
                    fix_actions: Some(vec![FixAction {
                        kind: "rescan".into(),
                        label: None,
                    }]),
                }],
            );
        }
    };

    // 3. Probe auth (optional)
    let auth_status = if let Some(ref args) = def.auth_probe_args {
        let timeout = def.auth_probe_timeout_ms.unwrap_or(5000);
        probe::probe_auth(&bin_path, args, timeout).await
    } else {
        None
    };

    // 4. Probe models (live if available, fallback otherwise)
    let (models, models_source) = probe::probe_models(
        &bin_path,
        def.list_models_args.as_deref(),
        def.list_models_timeout_ms.unwrap_or(5000),
        &def.fallback_models,
    )
    .await;

    // 5. Probe capabilities via --help
    let capabilities = if def.help_args.is_some() || !def.capabilities.is_empty() {
        let timeout = def.help_probe_timeout_ms.unwrap_or(5000);
        probe::probe_capabilities(
            &bin_path,
            def.help_args.as_deref(),
            timeout,
            &def.capabilities,
        )
        .await
    } else {
        vec![]
    };

    // Build diagnostics from auth
    let mut final_diagnostics = diagnostics;
    match auth_status {
        Some(AuthStatus::Missing) => {
            final_diagnostics.push(AgentDiagnostic {
                kind: "auth_missing".into(),
                message: format!(
                    "{} is installed but not authenticated. Run login command.",
                    def.name
                ),
                fix_actions: Some(vec![
                    FixAction {
                        kind: "signIn".into(),
                        label: Some(format!("Sign in to {}", def.name)),
                    },
                    FixAction {
                        kind: "rescan".into(),
                        label: Some("Rescan".into()),
                    },
                ]),
            });
        }
        Some(AuthStatus::Expired) => {
            final_diagnostics.push(AgentDiagnostic {
                kind: "auth_expired".into(),
                message: format!(
                    "{} authentication has expired. Please re-authenticate.",
                    def.name
                ),
                fix_actions: Some(vec![
                    FixAction {
                        kind: "reAuth".into(),
                        label: Some(format!("Re-authenticate {}", def.name)),
                    },
                    FixAction {
                        kind: "rescan".into(),
                        label: Some("Rescan".into()),
                    },
                ]),
            });
        }
        _ => {}
    }

    DetectedAgent {
        id: def.id.clone(),
        name: def.name.clone(),
        bin: def.bin.clone(),
        available,
        path: Some(bin_path.to_string_lossy().to_string()),
        version,
        models,
        models_source,
        auth_status,
        auth_message: None,
        diagnostics: if final_diagnostics.is_empty() {
            None
        } else {
            Some(final_diagnostics)
        },
        stream_format: Some(def.stream_format.clone()),
        install_url: def.install_url.clone(),
        docs_url: def.docs_url.clone(),
        capabilities,
    }
}

fn make_unavailable(def: &AgentDef, diagnostics: Vec<AgentDiagnostic>) -> DetectedAgent {
    DetectedAgent {
        id: def.id.clone(),
        name: def.name.clone(),
        bin: def.bin.clone(),
        available: false,
        path: None,
        version: None,
        models: def.fallback_models.clone(),
        models_source: ModelsSource::Fallback,
        auth_status: None,
        auth_message: None,
        diagnostics: Some(diagnostics),
        stream_format: Some(def.stream_format.clone()),
        install_url: def.install_url.clone(),
        docs_url: def.docs_url.clone(),
        capabilities: vec![],
    }
}
