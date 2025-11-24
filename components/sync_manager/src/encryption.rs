//! Encryption for synced data

use ring::{aead, pbkdf2, rand};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

use crate::{SyncError, SyncResult};

/// Number of PBKDF2 iterations for key derivation
const PBKDF2_ITERATIONS: u32 = 100_000;

/// Length of derived key in bytes (256 bits)
const KEY_LENGTH: usize = 32;

/// Length of nonce for AES-GCM (96 bits)
const NONCE_LENGTH: usize = 12;

/// Encryption key for sync data
#[derive(Clone)]
pub struct EncryptionKey {
    key_bytes: Vec<u8>,
}

impl EncryptionKey {
    /// Derive an encryption key from a password and email (as salt)
    pub fn derive_from_password(password: &str, email: &str) -> Self {
        let mut key_bytes = vec![0u8; KEY_LENGTH];

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
            email.as_bytes(),
            password.as_bytes(),
            &mut key_bytes,
        );

        Self { key_bytes }
    }

    /// Create from raw key bytes
    pub fn from_bytes(bytes: Vec<u8>) -> SyncResult<Self> {
        if bytes.len() != KEY_LENGTH {
            return Err(SyncError::EncryptionError(format!(
                "Invalid key length: expected {}, got {}",
                KEY_LENGTH,
                bytes.len()
            )));
        }
        Ok(Self { key_bytes: bytes })
    }

    /// Get the raw key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_bytes
    }
}

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionKey")
            .field("key_bytes", &"[REDACTED]")
            .finish()
    }
}

/// Handles encryption and decryption of sync data
#[derive(Debug)]
pub struct SyncEncryption {
    key: EncryptionKey,
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext (base64 encoded)
    pub ciphertext: String,

    /// The nonce used for encryption (base64 encoded)
    pub nonce: String,

    /// Version of encryption scheme used
    pub version: u8,
}

impl SyncEncryption {
    /// Create a new encryption handler with the given key
    pub fn new(key: EncryptionKey) -> Self {
        Self { key }
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> SyncResult<EncryptedData> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        // Generate random nonce
        let rng = rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        rand::SecureRandom::fill(&rng, &mut nonce_bytes)
            .map_err(|_| SyncError::EncryptionError("Failed to generate nonce".to_string()))?;

        // Create unbound key
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &self.key.key_bytes)
            .map_err(|_| SyncError::EncryptionError("Failed to create encryption key".to_string()))?;

        // Create sealing key with nonce
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        let sealing_key = aead::LessSafeKey::new(unbound_key);

        // Encrypt
        let mut in_out = plaintext.to_vec();
        sealing_key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| SyncError::EncryptionError("Encryption failed".to_string()))?;

        Ok(EncryptedData {
            ciphertext: STANDARD.encode(&in_out),
            nonce: STANDARD.encode(nonce_bytes),
            version: 1,
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> SyncResult<Vec<u8>> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        if encrypted.version != 1 {
            return Err(SyncError::EncryptionError(format!(
                "Unsupported encryption version: {}",
                encrypted.version
            )));
        }

        // Decode base64
        let ciphertext = STANDARD
            .decode(&encrypted.ciphertext)
            .map_err(|e| SyncError::EncryptionError(format!("Invalid ciphertext: {}", e)))?;

        let nonce_bytes = STANDARD
            .decode(&encrypted.nonce)
            .map_err(|e| SyncError::EncryptionError(format!("Invalid nonce: {}", e)))?;

        if nonce_bytes.len() != NONCE_LENGTH {
            return Err(SyncError::EncryptionError("Invalid nonce length".to_string()));
        }

        // Create opening key
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &self.key.key_bytes)
            .map_err(|_| SyncError::EncryptionError("Failed to create decryption key".to_string()))?;

        let mut nonce_array = [0u8; NONCE_LENGTH];
        nonce_array.copy_from_slice(&nonce_bytes);
        let nonce = aead::Nonce::assume_unique_for_key(nonce_array);
        let opening_key = aead::LessSafeKey::new(unbound_key);

        // Decrypt
        let mut in_out = ciphertext;
        let decrypted = opening_key
            .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| SyncError::EncryptionError("Decryption failed".to_string()))?;

        Ok(decrypted.to_vec())
    }

    /// Encrypt a JSON-serializable value
    pub fn encrypt_json<T: Serialize>(&self, value: &T) -> SyncResult<EncryptedData> {
        let json = serde_json::to_vec(value)?;
        self.encrypt(&json)
    }

    /// Decrypt to a JSON-deserializable value
    pub fn decrypt_json<T: for<'de> Deserialize<'de>>(&self, encrypted: &EncryptedData) -> SyncResult<T> {
        let decrypted = self.decrypt(encrypted)?;
        serde_json::from_slice(&decrypted).map_err(|e| SyncError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let key1 = EncryptionKey::derive_from_password("password123", "user@example.com");
        let key2 = EncryptionKey::derive_from_password("password123", "user@example.com");

        assert_eq!(key1.as_bytes(), key2.as_bytes());
        assert_eq!(key1.as_bytes().len(), KEY_LENGTH);
    }

    #[test]
    fn test_different_passwords() {
        let key1 = EncryptionKey::derive_from_password("password1", "user@example.com");
        let key2 = EncryptionKey::derive_from_password("password2", "user@example.com");

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_emails() {
        let key1 = EncryptionKey::derive_from_password("password", "user1@example.com");
        let key2 = EncryptionKey::derive_from_password("password", "user2@example.com");

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionKey::derive_from_password("test_password", "test@example.com");
        let encryption = SyncEncryption::new(key);

        let plaintext = b"Hello, World! This is secret data.";
        let encrypted = encryption.encrypt(plaintext).unwrap();

        assert!(!encrypted.ciphertext.is_empty());
        assert!(!encrypted.nonce.is_empty());
        assert_eq!(encrypted.version, 1);

        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_json() {
        let key = EncryptionKey::derive_from_password("test", "test@test.com");
        let encryption = SyncEncryption::new(key);

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            field1: String,
            field2: i32,
        }

        let data = TestData {
            field1: "test value".to_string(),
            field2: 42,
        };

        let encrypted = encryption.encrypt_json(&data).unwrap();
        let decrypted: TestData = encryption.decrypt_json(&encrypted).unwrap();

        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let result = EncryptionKey::from_bytes(vec![0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = EncryptionKey::derive_from_password("password1", "test@test.com");
        let key2 = EncryptionKey::derive_from_password("password2", "test@test.com");

        let encryption1 = SyncEncryption::new(key1);
        let encryption2 = SyncEncryption::new(key2);

        let encrypted = encryption1.encrypt(b"secret").unwrap();
        let result = encryption2.decrypt(&encrypted);

        assert!(result.is_err());
    }
}
