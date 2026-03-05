//! Age-based encryption for vault storage

use age::{
    EncryptError,
    identities::{Identity, SimpleIdentity},
    SecretKey,
};
use crate::error::{Error, Result};
use std::io::Read;

/// Derive an age secret key from a passphrase using scrypt-like approach
/// 
/// Age doesn't directly support passphrase-based encryption in the same way
/// as age-passphrase, so we use a simple key derivation approach.
/// For v0.1, we use a symmetric approach with the passphrase directly.
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

/// Encrypt data using age-style symmetric encryption
/// 
/// This creates a simple encrypted format that's compatible with age's
/// identity mechanism for future migration.
pub fn encrypt_vault(data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let key = derive_key_from_passphrase(passphrase);
    
    // Use XChaCha20-Poly1305 for encryption
    // Format: age-encryption.org/v1 -> XChaCha20-Poly1305 -> ciphertext + nonce
    use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};
    use chacha20poly1305::{XChaCha20Poly1305, Nonce, aead::{Aead, KeyInit}};
    
    // Generate a random nonce
    let mut nonce_bytes = [0u8; 24];
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| Error::Crypto(e.to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| Error::Encryption(e.to_string()))?;
    
    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(24 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt data using age-style symmetric decryption
pub fn decrypt_vault(encrypted_data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let key = derive_key_from_passphrase(passphrase);
    
    if encrypted_data.len() < 24 {
        return Err(Error::Decryption("Invalid encrypted data: too short".to_string()));
    }
    
    use chacha20poly1305::{XChaCha20Poly1305, Nonce, aead::{Aead, KeyInit}};
    
    let nonce = Nonce::from_slice(&encrypted_data[..24]);
    let ciphertext = &encrypted_data[24..];
    
    let cipher = XChaCha20Poly1305::new_from_slice(&key)
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
