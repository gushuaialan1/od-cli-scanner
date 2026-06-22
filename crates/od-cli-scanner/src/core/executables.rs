use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use which::which;

/// Resolve the executable path for an agent definition.
/// Priority: env override > PATH search > fallback bins
pub fn resolve_executable(
    def: &super::types::AgentDef,
    configured_env: &HashMap<String, String>,
) -> Option<PathBuf> {
    // 1. Check *_BIN env override from configured env
    if let Some(env_key) = &def.bin_env_key {
        if let Some(override_path) = configured_env.get(env_key) {
            let expanded = expand_home(override_path);
            if is_executable(&expanded) {
                return Some(expanded);
            }
        }
    }

    // 2. Check process env as fallback
    if let Some(env_key) = &def.bin_env_key {
        if let Ok(override_path) = env::var(env_key) {
            let expanded = expand_home(&override_path);
            if is_executable(&expanded) {
                return Some(expanded);
            }
        }
    }

    // 3. Try primary bin via PATH
    if let Ok(path) = which(&def.bin) {
        return Some(path);
    }

    // 4. Try fallback bins
    for fallback in &def.fallback_bins {
        if let Ok(path) = which(fallback) {
            return Some(path);
        }
    }

    None
}

/// Expand ~ to home directory
fn expand_home(path: &str) -> PathBuf {
    if path.starts_with("~/") || path.starts_with("~\\") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Check if a path is an executable file
fn is_executable(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let mode = metadata.permissions().mode();
            return mode & 0o111 != 0;
        }
    }

    #[cfg(windows)]
    {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        // Check PATHEXT for additional executable extensions
        let path_ext = env::var("PATHEXT")
            .unwrap_or(".COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC;.PS1".to_string());
        let valid_exts: Vec<String> = path_ext
            .split(';')
            .map(|s| s.trim().to_lowercase())
            .collect();
        return valid_exts.contains(&ext);
    }

    #[cfg(not(any(unix, windows)))]
    {
        return true;
    }

    #[allow(unreachable_code)]
    false
}

/// Get toolchain directories to append to PATH
pub fn toolchain_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join(".cargo/bin"));
        dirs.push(home.join(".bun/bin"));

        #[cfg(target_os = "macos")]
        {
            dirs.push(PathBuf::from("/opt/homebrew/bin"));
            dirs.push(PathBuf::from("/usr/local/bin"));
        }
    }

    dirs.into_iter().filter(|d| d.exists()).collect()
}
