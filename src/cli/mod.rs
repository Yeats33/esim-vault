//! Command-line interface for esim-vault

mod commands;

pub use commands::{build_cli, run_cli};

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "esimvault")]
#[command(version = "0.1.0")]
#[command(about = "Offline-first eSIM wallet/manager TUI tool", long_about = None)]
pub struct Cli {
    /// Vault file path
    #[arg(short, long, value_name = "PATH")]
    pub vault: Option<PathBuf>,

    /// Passphrase (will prompt if not provided)
    #[arg(short, long, value_name = "PASSPHRASE")]
    pub passphrase: Option<String>,

    /// Read passphrase from stdin
    #[arg(long)]
    pub pass_stdin: bool,

    /// Command to run
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init {
        /// Vault file path
        #[arg(short, long, value_name = "PATH")]
        vault: Option<PathBuf>,
    },

    /// Add a new eSIM profile
    Add {
        /// LPA payload text
        #[arg(short, long, value_name = "LPA_PAYLOAD")]
        text: Option<String>,

        /// Profile label
        #[arg(short, long, value_name = "LABEL")]
        label: Option<String>,

        /// Region tags (can be specified multiple times)
        #[arg(short, long, value_name = "TAG")]
        tag: Vec<String>,
    },

    /// List all profiles
    List {
        /// Filter by status
        #[arg(short, long, value_name = "STATUS")]
        status: Option<String>,

        /// Filter by tag
        #[arg(short, long, value_name = "TAG")]
        tag: Vec<String>,

        /// Search query
        #[arg(short, long, value_name = "QUERY")]
        search: Option<String>,
    },

    /// Show profile details
    Show {
        /// Profile ID
        #[arg(value_name = "ID")]
        id: String,

        /// Reveal sensitive data (will show warning)
        #[arg(long)]
        reveal: bool,
    },

    /// Mark profile status
    Mark {
        /// Profile ID
        #[arg(value_name = "ID")]
        id: String,

        /// Mark as unused
        #[arg(long)]
        unused: bool,

        /// Mark as used
        #[arg(long)]
        used: bool,

        /// Mark as expired
        #[arg(long)]
        expired: bool,
    },

    /// Generate QR code
    Qr {
        /// Profile ID
        #[arg(value_name = "ID")]
        id: String,

        /// Output file path
        #[arg(short, long, value_name = "PATH")]
        out: Option<PathBuf>,
    },

    /// Start TUI
    Tui {
        /// Vault file path
        #[arg(short, long, value_name = "PATH")]
        vault: Option<PathBuf>,
    },

    /// Edit profile tags
    Edit {
        /// Profile ID
        #[arg(value_name = "ID")]
        id: String,

        /// New label
        #[arg(long, value_name = "LABEL")]
        label: Option<String>,

        /// Add tags
        #[arg(long, value_name = "TAG")]
        add_tag: Vec<String>,

        /// Remove tags
        #[arg(long, value_name = "TAG")]
        remove_tag: Vec<String>,

        /// Set notes
        #[arg(long, value_name = "NOTES")]
        notes: Option<String>,
    },
}
