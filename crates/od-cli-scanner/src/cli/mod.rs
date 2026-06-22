use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "od-scan")]
#[command(about = "Detect installed AI coding agents on your system")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(short, long, global = true, default_value = "json")]
    pub format: OutputFormat,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Only show available agents
    #[arg(long, global = true)]
    pub available_only: bool,

    /// Filter by agent ID (comma-separated)
    #[arg(long, global = true)]
    pub filter: Option<String>,

    /// Timeout multiplier for slow systems
    #[arg(long, global = true, default_value = "1.0")]
    pub timeout_multiplier: f64,

    /// Verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan for all configured agents
    Scan {
        /// Custom config file (JSON with agent definitions)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Show built-in agent definitions
    ListDefs,
    /// Check a specific binary
    Check {
        /// Binary name or path
        binary: String,
        /// Arguments to pass (e.g., --version)
        #[arg(last = true)]
        args: Vec<String>,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
}
