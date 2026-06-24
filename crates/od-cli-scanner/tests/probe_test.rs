use od_cli_scanner::core::probe::{
    probe_auth, probe_capabilities, probe_models, probe_version, NotInvocableCause, ProbeError,
};
use od_cli_scanner::core::types::{AuthStatus, ModelOption, ModelsSource};
use std::fs;

#[cfg(unix)]
#[tokio::test]
async fn probe_version_success() {
    let result = probe_version(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "echo 'v1.0.0'".to_string()],
        5000,
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("1.0.0".to_string()));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_version_nonzero_exit_returns_none() {
    // A command that exits with non-zero but the binary is still invocable
    let result = probe_version(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "exit 1".to_string()],
        5000,
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[cfg(unix)]
#[tokio::test]
async fn probe_version_timeout() {
    // Sleep longer than the timeout
    let result = probe_version(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "sleep 5".to_string()],
        100,
    )
    .await;
    assert!(matches!(result, Err(ProbeError::Timeout)));
}

#[tokio::test]
async fn probe_version_not_found() {
    let result = probe_version(
        std::path::Path::new("/nonexistent/path/to/binary"),
        &["--version".to_string()],
        5000,
    )
    .await;
    assert!(matches!(
        result,
        Err(ProbeError::NotInvocable(NotInvocableCause::MissingTarget))
    ));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_version_not_executable() {
    let tmpdir = tempfile::tempdir().unwrap();
    let fake_bin = tmpdir.path().join("not_exec");
    fs::write(&fake_bin, "#!/bin/sh\n").unwrap();
    // No execute permission

    let result = probe_version(&fake_bin, &["--version".to_string()], 5000).await;
    assert!(matches!(
        result,
        Err(ProbeError::NotInvocable(NotInvocableCause::NotExecutable))
    ));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_auth_ok() {
    // Simulate successful auth by outputting nothing suspicious
    let result = probe_auth(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "echo 'authenticated'".to_string()],
        5000,
    )
    .await;
    assert_eq!(result, Some(AuthStatus::Ok));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_auth_missing() {
    let result = probe_auth(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "echo 'not authenticated'".to_string()],
        5000,
    )
    .await;
    assert_eq!(result, Some(AuthStatus::Missing));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_auth_login_required() {
    let result = probe_auth(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "echo 'login required'".to_string()],
        5000,
    )
    .await;
    assert_eq!(result, Some(AuthStatus::Missing));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_auth_no_valid_credentials() {
    let result = probe_auth(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "echo 'no valid credentials'".to_string()],
        5000,
    )
    .await;
    assert_eq!(result, Some(AuthStatus::Missing));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_auth_unknown_on_failure() {
    let result = probe_auth(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "exit 1".to_string()],
        5000,
    )
    .await;
    assert_eq!(result, Some(AuthStatus::Unknown));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_auth_timeout_returns_none() {
    let result = probe_auth(
        std::path::Path::new("/bin/sh"),
        &["-c".to_string(), "sleep 5".to_string()],
        100,
    )
    .await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn probe_models_returns_fallback() {
    let fallback = vec![
        ModelOption {
            id: "model1".to_string(),
            label: "Model 1".to_string(),
        },
        ModelOption {
            id: "model2".to_string(),
            label: "Model 2".to_string(),
        },
    ];

    let (models, source) =
        probe_models(std::path::Path::new("/bin/sh"), None, 5000, &fallback).await;

    assert_eq!(models, fallback);
    assert_eq!(source, ModelsSource::Fallback);
}

#[cfg(unix)]
#[tokio::test]
async fn probe_capabilities_detects_flags() {
    let help_text = "Usage: test [OPTIONS]\n  -m, --model    Select model\n  -p, --prompt   Prompt text\n  -f, --file     Input file\n  --stream       Stream output\n  -i, --interactive  Interactive mode\n  -b, --batch    Batch mode\n  -c, --config   Config file\n  -V, --verbose  Verbose output\n  -q, --quiet    Quiet mode\n  -o, --output   Output file\n  -d, --directory  Working directory\n  -w, --workspace  Workspace\n  --project      Project name\n  --auth         Auth token\n  --debug        Debug mode\n  --dry-run      Dry run\n  --json         JSON output\n  --markdown     Markdown output";
    let result = probe_capabilities(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), format!("echo '{}'", help_text)]),
        5000,
        &[],
    )
    .await;

    assert!(result.contains(&"model".to_string()));
    assert!(result.contains(&"prompt".to_string()));
    assert!(result.contains(&"file".to_string()));
    assert!(result.contains(&"stream".to_string()));
    assert!(result.contains(&"interactive".to_string()));
    assert!(result.contains(&"batch".to_string()));
    assert!(result.contains(&"config".to_string()));
    assert!(result.contains(&"verbose".to_string()));
    assert!(result.contains(&"quiet".to_string()));
    assert!(result.contains(&"output".to_string()));
    assert!(result.contains(&"directory".to_string()));
    assert!(result.contains(&"project".to_string()));
    assert!(result.contains(&"workspace".to_string()));
    assert!(result.contains(&"auth".to_string()));
    assert!(result.contains(&"debug".to_string()));
    assert!(result.contains(&"dry-run".to_string()));
    assert!(result.contains(&"json".to_string()));
    assert!(result.contains(&"markdown".to_string()));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_capabilities_detects_expected_caps() {
    let help_text = "Usage: test [OPTIONS]\n  --custom-flag  Custom flag";
    let result = probe_capabilities(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), format!("echo '{}'", help_text)]),
        5000,
        &["custom-flag".to_string()],
    )
    .await;

    assert!(result.contains(&"custom-flag".to_string()));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_capabilities_returns_empty_on_failure() {
    let result = probe_capabilities(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), "sleep 5".to_string()]),
        100,
        &[],
    )
    .await;

    assert!(result.is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn probe_capabilities_deduplicates() {
    let help_text = "--model\n--model\n--verbose";
    let result = probe_capabilities(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), format!("echo '{}'", help_text)]),
        5000,
        &[],
    )
    .await;

    assert_eq!(result.len(), 2);
    assert!(result.contains(&"model".to_string()));
    assert!(result.contains(&"verbose".to_string()));
}

#[tokio::test]
async fn probe_capabilities_not_found_returns_empty() {
    let result = probe_capabilities(
        std::path::Path::new("/nonexistent/path/to/binary"),
        Some(&["--help".to_string()]),
        5000,
        &[],
    )
    .await;

    assert!(result.is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn probe_models_parses_json_array() {
    let fallback = vec![ModelOption {
        id: "fallback".to_string(),
        label: "Fallback".to_string(),
    }];

    let (models, source) = probe_models(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), "echo '[{\"id\":\"gpt-4\",\"label\":\"GPT-4\"}]'".to_string()]),
        5000,
        &fallback,
    )
    .await;

    assert_eq!(source, ModelsSource::Live);
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "gpt-4");
    assert_eq!(models[0].label, "GPT-4");
}

#[cfg(unix)]
#[tokio::test]
async fn probe_models_parses_json_object_with_models_key() {
    let fallback = vec![ModelOption {
        id: "fallback".to_string(),
        label: "Fallback".to_string(),
    }];

    let (models, source) = probe_models(
        std::path::Path::new("/bin/sh"),
        Some(&[
            "-c".to_string(),
            "echo '{\"models\":[{\"id\":\"claude-sonnet\",\"label\":\"Sonnet\"}]}'".to_string(),
        ]),
        5000,
        &fallback,
    )
    .await;

    assert_eq!(source, ModelsSource::Live);
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "claude-sonnet");
    assert_eq!(models[0].label, "Sonnet");
}

#[cfg(unix)]
#[tokio::test]
async fn probe_models_parses_plain_text() {
    let fallback = vec![ModelOption {
        id: "fallback".to_string(),
        label: "Fallback".to_string(),
    }];

    let (models, source) = probe_models(
        std::path::Path::new("/bin/sh"),
        Some(&[
            "-c".to_string(),
            "printf 'gpt-4  GPT-4\ngpt-3.5  GPT-3.5\n'".to_string(),
        ]),
        5000,
        &fallback,
    )
    .await;

    assert_eq!(source, ModelsSource::Live);
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "gpt-4");
    assert_eq!(models[0].label, "GPT-4");
    assert_eq!(models[1].id, "gpt-3.5");
    assert_eq!(models[1].label, "GPT-3.5");
}

#[cfg(unix)]
#[tokio::test]
async fn probe_models_falls_back_on_empty_output() {
    let fallback = vec![ModelOption {
        id: "fallback".to_string(),
        label: "Fallback".to_string(),
    }];

    let (models, source) = probe_models(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), "echo ''".to_string()]),
        5000,
        &fallback,
    )
    .await;

    assert_eq!(source, ModelsSource::Fallback);
    assert_eq!(models, fallback);
}

#[cfg(unix)]
#[tokio::test]
async fn probe_models_falls_back_on_nonzero_exit() {
    let fallback = vec![ModelOption {
        id: "fallback".to_string(),
        label: "Fallback".to_string(),
    }];

    let (models, source) = probe_models(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), "exit 1".to_string()]),
        5000,
        &fallback,
    )
    .await;

    assert_eq!(source, ModelsSource::Fallback);
    assert_eq!(models, fallback);
}

#[cfg(unix)]
#[tokio::test]
async fn probe_models_falls_back_on_timeout() {
    let fallback = vec![ModelOption {
        id: "fallback".to_string(),
        label: "Fallback".to_string(),
    }];

    let (models, source) = probe_models(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), "sleep 5".to_string()]),
        100,
        &fallback,
    )
    .await;

    assert_eq!(source, ModelsSource::Fallback);
    assert_eq!(models, fallback);
}

#[cfg(unix)]
#[tokio::test]
async fn probe_models_parses_plain_text_without_label() {
    let fallback = vec![ModelOption {
        id: "fallback".to_string(),
        label: "Fallback".to_string(),
    }];

    let (models, source) = probe_models(
        std::path::Path::new("/bin/sh"),
        Some(&["-c".to_string(), "printf 'model-a\nmodel-b\n'".to_string()]),
        5000,
        &fallback,
    )
    .await;

    assert_eq!(source, ModelsSource::Live);
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "model-a");
    assert_eq!(models[0].label, "model-a");
    assert_eq!(models[1].id, "model-b");
    assert_eq!(models[1].label, "model-b");
}
