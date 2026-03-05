//! Age-based encryption for vault storage

use crate::error::{Error, Result};

/// Derive a secret key from a passphrase using SHA-256
fn derive_key_from_passphrase(passphrase: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.update(b"esim-vault-v1");  // Salt/version identifier
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Encrypt data using AES-GCM
pub fn encrypt_vault(data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    
    let key = derive_key_from_passphrase(passphrase);
    
    // Generate a random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| Error::Crypto(e.to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| Error::Encryption(e.to_string()))?;
    
    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt data using AES-GCM
pub fn decrypt_vault(encrypted_data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    
    let key = derive_key_from_passphrase(passphrase);
    
    if encrypted_data.len() < 12 {
        return Err(Error::Decryption("Invalid encrypted data: too short".to_string()));
    }
    
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let ciphertext = &encrypted_data[12..];
    
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| Error::Decryption("Decryption failed: wrong passphrase or corrupted data".to_string()))?;
    
    Ok(plaintext)
}

/// Prompt for passphrase (used by CLI)
pub fn read_passphrase() -> Result<String> {
    use std::io::{self, Write};
    
    // Try to read from environment variable first
    if let Ok(passphrase) = std::env::var("ESIMVAULT_PASSPHRASE") {
        if !passphrase.is_empty() {
            return Ok(passphrase);
        }
    }
    
    // Read from stdin if --pass-stdin is used
    // This is handled by clap, so we just do a simple prompt here
    
    print!("Enter vault passphrase: ");
    io::stdout().flush()?;
    
    // Use rpassword for secure input
    match rpassword::read_password() {
        Ok(pass) => Ok(pass),
        Err(_) => {
            // Fallback to simple line read
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            Ok(line.trim().to_string())
        }
    }
}

/// Read passphrase from stdin (for CLI flag)
pub fn read_passphrase_from_stdin() -> Result<String> {
    use std::io::{self, Read};
    
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    
    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let data = b"Hello, World!";
        let passphrase = "test-password-123";
        
        let encrypted = encrypt_vault(data, passphrase).unwrap();
        let decrypted = decrypt_vault(&encrypted, passphrase).unwrap();
        
        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_wrong_passphrase() {
        let data = b"Secret data";
        let passphrase = "correct-password";
        
        let encrypted = encrypt_vault(data, passphrase).unwrap();
        let result = decrypt_vault(&encrypted, "wrong-password");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation() {
        let key1 = derive_key_from_passphrase("test");
        let key2 = derive_key_from_passphrase("test");
        let key3 = derive_key_from_passphrase("different");
        
        assert_eq!(key1, key2);  // Same passphrase -> same key
        assert_ne!(key1, key3);  // Different passphrase -> different key
    }
}
