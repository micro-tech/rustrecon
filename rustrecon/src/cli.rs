use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initializes configuration files
    Init {
        /// Path to create the config file (uses user config directory by default)
        #[clap(short, long)]
        config_path: Option<String>,
    },
    /// Tests the LLM API connection
    Test,
    /// Scans a specified crate
    Scan {
        /// Path to the crate to scan
        #[clap(value_parser)]
        crate_path: String,
        /// Output format for the report (json, markdown, condensed, summary)
        #[clap(short, long, default_value = "markdown")]
        format: String,
        /// Output file for the report
        #[clap(short, long)]
        output: Option<String>,
        /// Enable dependency scanning for supply chain security
        #[clap(short = 'd', long, default_value = "true")]
        scan_dependencies: bool,
        /// Skip dependency scanning (code only)
        #[clap(long)]
        skip_dependencies: bool,
    },
}
