use clap::Parser;
use od_cli_scanner::cli::{Cli, Commands, OutputFormat};

#[test]
fn cli_scan_subcommand_parsing() {
    let args = vec!["od-scan", "scan"];
    let cli = Cli::parse_from(args);
    assert!(matches!(cli.command, Commands::Scan { config: None }));
    assert!(matches!(cli.format, OutputFormat::Json));
    assert!(!cli.pretty);
    assert!(!cli.available_only);
    assert!(!cli.verbose);
}

#[test]
fn cli_scan_with_config() {
    let args = vec!["od-scan", "scan", "--config", "/tmp/agents.json"];
    let cli = Cli::parse_from(args);
    assert!(matches!(
        cli.command,
        Commands::Scan {
            config: Some(ref p)
        } if p == std::path::Path::new("/tmp/agents.json")
    ));
}

#[test]
fn cli_list_defs_subcommand() {
    let args = vec!["od-scan", "list-defs"];
    let cli = Cli::parse_from(args);
    assert!(matches!(cli.command, Commands::ListDefs));
}

#[test]
fn cli_check_subcommand() {
    let args = vec!["od-scan", "check", "mybin", "--", "--version"];
    let cli = Cli::parse_from(args);
    assert!(matches!(
        cli.command,
        Commands::Check {
            binary,
            args,
        } if binary == "mybin" && args == vec!["--version"]
    ));
}

#[test]
fn cli_format_table() {
    let args = vec!["od-scan", "--format", "table", "scan"];
    let cli = Cli::parse_from(args);
    assert!(matches!(cli.format, OutputFormat::Table));
}

#[test]
fn cli_format_csv() {
    let args = vec!["od-scan", "--format", "csv", "scan"];
    let cli = Cli::parse_from(args);
    assert!(matches!(cli.format, OutputFormat::Csv));
}

#[test]
fn cli_pretty_flag() {
    let args = vec!["od-scan", "--pretty", "scan"];
    let cli = Cli::parse_from(args);
    assert!(cli.pretty);
}

#[test]
fn cli_available_only_flag() {
    let args = vec!["od-scan", "--available-only", "scan"];
    let cli = Cli::parse_from(args);
    assert!(cli.available_only);
}

#[test]
fn cli_filter_option() {
    let args = vec!["od-scan", "--filter", "claude,codex", "scan"];
    let cli = Cli::parse_from(args);
    assert_eq!(cli.filter, Some("claude,codex".to_string()));
}

#[test]
fn cli_timeout_multiplier() {
    let args = vec!["od-scan", "--timeout-multiplier", "2.5", "scan"];
    let cli = Cli::parse_from(args);
    assert!((cli.timeout_multiplier - 2.5).abs() < f64::EPSILON);
}

#[test]
fn cli_verbose_flag() {
    let args = vec!["od-scan", "--verbose", "scan"];
    let cli = Cli::parse_from(args);
    assert!(cli.verbose);
}

#[test]
fn cli_short_verbose_flag() {
    let args = vec!["od-scan", "-v", "scan"];
    let cli = Cli::parse_from(args);
    assert!(cli.verbose);
}

#[test]
fn cli_combined_flags() {
    let args = vec![
        "od-scan",
        "-v",
        "--format",
        "table",
        "--pretty",
        "--available-only",
        "--filter",
        "claude",
        "scan",
    ];
    let cli = Cli::parse_from(args);
    assert!(cli.verbose);
    assert!(matches!(cli.format, OutputFormat::Table));
    assert!(cli.pretty);
    assert!(cli.available_only);
    assert_eq!(cli.filter, Some("claude".to_string()));
}
