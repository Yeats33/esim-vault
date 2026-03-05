//! Vault storage with age encryption

mod crypto;

pub use crypto::{decrypt_vault, encrypt_vault};

#[allow(unused_imports)]
use crate::core::{Profile, Vault};
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;

/// Load a vault from an encrypted file
pub fn load_vault<P: AsRef<Path>>(path: P, passphrase: &str) -> Result<Vault> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(Error::Vault(format!(
            "Vault file not found: {}",
            path.display()
        )));
    }

    let encrypted_data = fs::read(path)?;
    let decrypted = decrypt_vault(&encrypted_data, passphrase)?;
    let vault: Vault = serde_json::from_slice(&decrypted)?;

    Ok(vault)
}

/// Save a vault to an encrypted file
pub fn save_vault<P: AsRef<Path>>(vault: &Vault, path: P, passphrase: &str) -> Result<()> {
    let path = path.as_ref();

    let json = serde_json::to_vec(vault)?;
    let encrypted = encrypt_vault(&json, passphrase)?;

    // Write to a temp file first, then atomically rename
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &encrypted)?;
    fs::rename(&temp_path, path)?;

    Ok(())
}

/// Create a new empty vault at the given path
pub fn create_vault<P: AsRef<Path>>(path: P, passphrase: &str) -> Result<Vault> {
    let vault = Vault::new();
    save_vault(&vault, path, passphrase)?;
    Ok(vault)
}

/// Check if a vault file exists
#[allow(dead_code)]
pub fn vault_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_vault_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test.esimvault");

        // Create a vault with a profile
        let vault = Vault::new();
        let mut profile = Profile::new("Test".to_string(), "LPA:1$sm-dp$ABC123".to_string());
        profile.add_tag("Japan".to_string());
        let mut vault = vault;
        vault.add_profile(profile);

        // Save it
        save_vault(&vault, &vault_path, "test-password").unwrap();

        // Load it
        let loaded = load_vault(&vault_path, "test-password").unwrap();

        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].label, "Test");
        assert_eq!(loaded.profiles[0].region_tags, vec!["Japan"]);
    }

    #[test]
    fn test_wrong_passphrase() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("test.esimvault");

        let vault = Vault::new();
        save_vault(&vault, &vault_path, "correct-password").unwrap();

        let result = load_vault(&vault_path, "wrong-password");
        assert!(result.is_err());
    }
}
