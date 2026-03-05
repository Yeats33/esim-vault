//! Error types for esim-vault

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Vault error: {0}")]
    Vault(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid LPA payload: {0}")]
    InvalidLpa(String),

    #[error("QR error: {0}")]
    Qr(String),

    #[error("TUI error: {0}")]
    Tui(String),
}
