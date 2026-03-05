//! Command-line interface for esim-vault

mod commands;

pub use commands::{build_cli, run_cli, get_passphrase, get_vault_path};
