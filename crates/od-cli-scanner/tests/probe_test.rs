use od_cli_scanner::core::probe::{
    probe_auth, probe_models, probe_version, NotInvocableCause, ProbeError,
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
