use regex::Regex;

/// Parse version string from CLI output.
/// Covers common formats like:
/// - "claude 2.1.0"
/// - "kimi 0.18.0"
/// - "1.2.3"
/// - "version: 1.0.0-beta"
/// - "v2.0.1"
pub fn parse_version(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    // Try to find a semantic version pattern first
    let semver_re = Regex::new(r"v?(\d+\.\d+(?:\.\d+)?(?:-[\w.]+)?)").unwrap();
    if let Some(caps) = semver_re.captures(raw) {
        return Some(caps.get(1).unwrap().as_str().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_claude_version() {
        assert_eq!(parse_version("claude 2.1.0"), Some("2.1.0".to_string()));
    }

    #[test]
    fn parse_kimi_version() {
        assert_eq!(parse_version("kimi 0.18.0"), Some("0.18.0".to_string()));
    }

    #[test]
    fn parse_plain_semver() {
        assert_eq!(parse_version("1.2.3"), Some("1.2.3".to_string()));
    }

    #[test]
    fn parse_v_prefix() {
        assert_eq!(parse_version("v2.0.1"), Some("2.0.1".to_string()));
    }

    #[test]
    fn parse_with_beta() {
        assert_eq!(
            parse_version("version: 1.0.0-beta"),
            Some("1.0.0-beta".to_string())
        );
    }

    #[test]
    fn parse_multiline_first_line() {
        assert_eq!(
            parse_version("claude 2.1.0\nmore stuff"),
            Some("2.1.0".to_string())
        );
    }

    #[test]
    fn parse_empty_returns_none() {
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn parse_no_version_returns_none() {
        assert_eq!(parse_version("unknown"), None);
    }
}
