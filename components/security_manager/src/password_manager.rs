//! Password Manager Module
//!
//! Provides secure credential storage, password generation, and autofill functionality.
//!
//! # Features
//!
//! - Secure credential storage with AES-256-GCM encryption
//! - Master password derivation using Argon2id
//! - Configurable password generation
//! - Site-based autofill matching
//! - Import/export credentials (encrypted)
//! - Breach detection (checks against known compromised passwords)
//!
//! # Security Model
//!
//! - All passwords are encrypted at rest using AES-256-GCM
//! - Master password is never stored; only a derived key is used
//! - Key derivation uses Argon2id with recommended parameters
//! - Each credential has a unique nonce for encryption
//! - Memory is zeroed after use for sensitive data

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Errors that can occur during password manager operations
#[derive(Debug, Error)]
pub enum PasswordError {
    /// Vault is locked and requires unlocking
    #[error("Vault is locked. Please unlock with master password.")]
    VaultLocked,

    /// Invalid master password provided
    #[error("Invalid master password")]
    InvalidMasterPassword,

    /// Credential not found
    #[error("Credential not found: {0}")]
    CredentialNotFound(String),

    /// Encryption operation failed
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Key derivation failed
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Import/export error
    #[error("Import/export error: {0}")]
    ImportExportError(String),

    /// Password generation error
    #[error("Password generation error: {0}")]
    GenerationError(String),

    /// Breach detected
    #[error("Password found in breach database")]
    BreachDetected,
}

pub type Result<T> = std::result::Result<T, PasswordError>;

/// Unique identifier for credentials
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CredentialId(Uuid);

impl CredentialId {
    /// Create a new random CredentialId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CredentialId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CredentialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Encrypted string that holds ciphertext and nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedString {
    /// Base64-encoded ciphertext
    ciphertext: String,
    /// Base64-encoded nonce (12 bytes for AES-GCM)
    nonce: String,
}

impl EncryptedString {
    /// Create a new encrypted string
    fn new(ciphertext: Vec<u8>, nonce: Vec<u8>) -> Self {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        Self {
            ciphertext: engine.encode(ciphertext),
            nonce: engine.encode(nonce),
        }
    }

    /// Get the ciphertext bytes
    fn ciphertext_bytes(&self) -> Result<Vec<u8>> {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        engine
            .decode(&self.ciphertext)
            .map_err(|e| PasswordError::DecryptionFailed(e.to_string()))
    }

    /// Get the nonce bytes
    fn nonce_bytes(&self) -> Result<Vec<u8>> {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        engine
            .decode(&self.nonce)
            .map_err(|e| PasswordError::DecryptionFailed(e.to_string()))
    }
}

/// A stored credential with encrypted password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Unique identifier
    pub id: CredentialId,
    /// Site URL or domain
    pub site: String,
    /// Username or email
    pub username: String,
    /// Encrypted password
    pub password: EncryptedString,
    /// Optional notes (encrypted)
    pub notes: Option<EncryptedString>,
    /// When the credential was created
    pub created_at: DateTime<Utc>,
    /// When the credential was last modified
    pub modified_at: DateTime<Utc>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Whether this credential should be used for autofill
    pub autofill_enabled: bool,
}

/// Decrypted credential for use in memory
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct DecryptedCredential {
    /// Unique identifier (not sensitive, but included for convenience)
    #[zeroize(skip)]
    pub id: CredentialId,
    /// Site URL or domain
    #[zeroize(skip)]
    pub site: String,
    /// Username or email
    pub username: String,
    /// Decrypted password
    pub password: String,
    /// Decrypted notes
    pub notes: Option<String>,
}

/// Password generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    /// Length of generated password
    pub length: usize,
    /// Include uppercase letters (A-Z)
    pub uppercase: bool,
    /// Include lowercase letters (a-z)
    pub lowercase: bool,
    /// Include digits (0-9)
    pub digits: bool,
    /// Include special characters (!@#$%^&*...)
    pub special: bool,
    /// Exclude ambiguous characters (0, O, l, 1, I)
    pub exclude_ambiguous: bool,
    /// Custom characters to include
    pub custom_chars: Option<String>,
    /// Minimum number of each character type
    pub min_per_type: usize,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 16,
            uppercase: true,
            lowercase: true,
            digits: true,
            special: true,
            exclude_ambiguous: true,
            custom_chars: None,
            min_per_type: 1,
        }
    }
}

impl PasswordConfig {
    /// Create a simple config (letters and numbers only)
    pub fn simple(length: usize) -> Self {
        Self {
            length,
            uppercase: true,
            lowercase: true,
            digits: true,
            special: false,
            exclude_ambiguous: true,
            custom_chars: None,
            min_per_type: 1,
        }
    }

    /// Create a strong config (all character types)
    pub fn strong(length: usize) -> Self {
        Self {
            length,
            uppercase: true,
            lowercase: true,
            digits: true,
            special: true,
            exclude_ambiguous: false,
            custom_chars: None,
            min_per_type: 2,
        }
    }

    /// Create a PIN config (digits only)
    pub fn pin(length: usize) -> Self {
        Self {
            length,
            uppercase: false,
            lowercase: false,
            digits: true,
            special: false,
            exclude_ambiguous: false,
            custom_chars: None,
            min_per_type: 0,
        }
    }
}

/// Password generator
pub struct PasswordGenerator;

impl PasswordGenerator {
    const UPPERCASE: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &'static str = "abcdefghijklmnopqrstuvwxyz";
    const DIGITS: &'static str = "0123456789";
    const SPECIAL: &'static str = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    const AMBIGUOUS: &'static str = "0O1lI";

    /// Generate a password based on configuration
    pub fn generate(config: &PasswordConfig) -> Result<String> {
        if config.length == 0 {
            return Err(PasswordError::GenerationError(
                "Password length must be greater than 0".to_string(),
            ));
        }

        let mut charset = String::new();
        let mut required_chars: Vec<char> = Vec::new();
        let mut rng = rand::thread_rng();

        // Build charset and collect required characters
        if config.uppercase {
            let chars = Self::filter_ambiguous(Self::UPPERCASE, config.exclude_ambiguous);
            charset.push_str(&chars);
            for _ in 0..config.min_per_type {
                if let Some(c) = Self::random_char(&chars, &mut rng) {
                    required_chars.push(c);
                }
            }
        }

        if config.lowercase {
            let chars = Self::filter_ambiguous(Self::LOWERCASE, config.exclude_ambiguous);
            charset.push_str(&chars);
            for _ in 0..config.min_per_type {
                if let Some(c) = Self::random_char(&chars, &mut rng) {
                    required_chars.push(c);
                }
            }
        }

        if config.digits {
            let chars = Self::filter_ambiguous(Self::DIGITS, config.exclude_ambiguous);
            charset.push_str(&chars);
            for _ in 0..config.min_per_type {
                if let Some(c) = Self::random_char(&chars, &mut rng) {
                    required_chars.push(c);
                }
            }
        }

        if config.special {
            charset.push_str(Self::SPECIAL);
            for _ in 0..config.min_per_type {
                if let Some(c) = Self::random_char(Self::SPECIAL, &mut rng) {
                    required_chars.push(c);
                }
            }
        }

        if let Some(ref custom) = config.custom_chars {
            charset.push_str(custom);
        }

        if charset.is_empty() {
            return Err(PasswordError::GenerationError(
                "No character types selected".to_string(),
            ));
        }

        if required_chars.len() > config.length {
            return Err(PasswordError::GenerationError(format!(
                "Cannot satisfy minimum requirements ({} chars) with password length {}",
                required_chars.len(),
                config.length
            )));
        }

        // Generate remaining random characters
        let remaining = config.length - required_chars.len();
        let charset_chars: Vec<char> = charset.chars().collect();

        for _ in 0..remaining {
            let idx = rng.gen_range(0..charset_chars.len());
            required_chars.push(charset_chars[idx]);
        }

        // Shuffle the password
        use rand::seq::SliceRandom;
        required_chars.shuffle(&mut rng);

        Ok(required_chars.into_iter().collect())
    }

    fn filter_ambiguous(chars: &str, exclude: bool) -> String {
        if exclude {
            chars
                .chars()
                .filter(|c| !Self::AMBIGUOUS.contains(*c))
                .collect()
        } else {
            chars.to_string()
        }
    }

    fn random_char(chars: &str, rng: &mut impl Rng) -> Option<char> {
        let char_vec: Vec<char> = chars.chars().collect();
        if char_vec.is_empty() {
            None
        } else {
            Some(char_vec[rng.gen_range(0..char_vec.len())])
        }
    }
}

/// Encryption key derived from master password (zeroed on drop)
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
struct DerivedKey {
    key: [u8; 32],
}

impl DerivedKey {
    fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// Master password salt for key derivation
#[derive(Clone, Serialize, Deserialize)]
struct VaultMetadata {
    /// Salt for Argon2 key derivation
    salt: String,
    /// Hash of derived key for verification
    key_hash: String,
    /// Version of the vault format
    version: u32,
}

/// Encrypted credential store
pub struct CredentialStore {
    /// Stored credentials (encrypted)
    credentials: Arc<RwLock<HashMap<CredentialId, Credential>>>,
    /// Derived encryption key (only present when unlocked)
    derived_key: Arc<RwLock<Option<DerivedKey>>>,
    /// Vault metadata
    metadata: Arc<RwLock<Option<VaultMetadata>>>,
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(RwLock::new(HashMap::new())),
            derived_key: Arc::new(RwLock::new(None)),
            metadata: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the vault with a master password
    pub async fn initialize(&self, master_password: &str) -> Result<()> {
        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Derive key using Argon2id
        let key = self.derive_key(master_password, salt.as_str())?;

        // Hash the key for verification
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());

        // Store metadata
        let metadata = VaultMetadata {
            salt: salt.to_string(),
            key_hash,
            version: 1,
        };

        *self.metadata.write().await = Some(metadata);
        *self.derived_key.write().await = Some(key);

        Ok(())
    }

    /// Unlock the vault with master password
    pub async fn unlock(&self, master_password: &str) -> Result<()> {
        let metadata = self
            .metadata
            .read()
            .await
            .clone()
            .ok_or(PasswordError::InvalidMasterPassword)?;

        // Derive key
        let key = self.derive_key(master_password, &metadata.salt)?;

        // Verify key hash
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != metadata.key_hash {
            return Err(PasswordError::InvalidMasterPassword);
        }

        *self.derived_key.write().await = Some(key);
        Ok(())
    }

    /// Lock the vault (clear derived key from memory)
    pub async fn lock(&self) {
        let mut key = self.derived_key.write().await;
        if let Some(ref mut k) = *key {
            k.zeroize();
        }
        *key = None;
    }

    /// Check if the vault is unlocked
    pub async fn is_unlocked(&self) -> bool {
        self.derived_key.read().await.is_some()
    }

    /// Save a credential
    pub async fn save(&self, credential: Credential) -> Result<CredentialId> {
        if !self.is_unlocked().await {
            return Err(PasswordError::VaultLocked);
        }

        let id = credential.id;
        self.credentials.write().await.insert(id, credential);
        Ok(id)
    }

    /// Create and save a new credential
    pub async fn create(
        &self,
        site: String,
        username: String,
        password: &str,
        notes: Option<&str>,
    ) -> Result<CredentialId> {
        let key = self.get_key().await?;
        let now = Utc::now();

        let encrypted_password = self.encrypt_string(password, &key)?;
        let encrypted_notes = notes
            .map(|n| self.encrypt_string(n, &key))
            .transpose()?;

        let credential = Credential {
            id: CredentialId::new(),
            site,
            username,
            password: encrypted_password,
            notes: encrypted_notes,
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            autofill_enabled: true,
        };

        self.save(credential).await
    }

    /// Get a credential by ID
    pub async fn get(&self, id: CredentialId) -> Result<Credential> {
        if !self.is_unlocked().await {
            return Err(PasswordError::VaultLocked);
        }

        self.credentials
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| PasswordError::CredentialNotFound(id.to_string()))
    }

    /// Get a decrypted credential by ID
    pub async fn get_decrypted(&self, id: CredentialId) -> Result<DecryptedCredential> {
        let credential = self.get(id).await?;
        let key = self.get_key().await?;

        let password = self.decrypt_string(&credential.password, &key)?;
        let notes = credential
            .notes
            .as_ref()
            .map(|n| self.decrypt_string(n, &key))
            .transpose()?;

        Ok(DecryptedCredential {
            id: credential.id,
            site: credential.site,
            username: credential.username,
            password,
            notes,
        })
    }

    /// Find credentials matching a site (for autofill)
    pub async fn find_for_site(&self, site: &str) -> Vec<Credential> {
        if !self.is_unlocked().await {
            return Vec::new();
        }

        let credentials = self.credentials.read().await;
        let site_lower = site.to_lowercase();

        // Extract domain from site for matching
        let site_domain = Self::extract_domain(&site_lower);

        credentials
            .values()
            .filter(|c| {
                if !c.autofill_enabled {
                    return false;
                }

                let cred_domain = Self::extract_domain(&c.site.to_lowercase());

                // Match if domains are the same or one is a subdomain of the other
                cred_domain == site_domain
                    || site_domain.ends_with(&format!(".{}", cred_domain))
                    || cred_domain.ends_with(&format!(".{}", site_domain))
            })
            .cloned()
            .collect()
    }

    /// Delete a credential
    pub async fn delete(&self, id: CredentialId) -> Result<()> {
        if !self.is_unlocked().await {
            return Err(PasswordError::VaultLocked);
        }

        self.credentials
            .write()
            .await
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| PasswordError::CredentialNotFound(id.to_string()))
    }

    /// Update a credential's password
    pub async fn update_password(&self, id: CredentialId, new_password: &str) -> Result<()> {
        let key = self.get_key().await?;

        let mut credentials = self.credentials.write().await;
        let credential = credentials
            .get_mut(&id)
            .ok_or_else(|| PasswordError::CredentialNotFound(id.to_string()))?;

        credential.password = self.encrypt_string(new_password, &key)?;
        credential.modified_at = Utc::now();

        Ok(())
    }

    /// List all credentials (without decrypting passwords)
    pub async fn list(&self) -> Result<Vec<Credential>> {
        if !self.is_unlocked().await {
            return Err(PasswordError::VaultLocked);
        }

        Ok(self.credentials.read().await.values().cloned().collect())
    }

    /// Export credentials (encrypted with provided password)
    pub async fn export(&self, export_password: &str) -> Result<String> {
        if !self.is_unlocked().await {
            return Err(PasswordError::VaultLocked);
        }

        let credentials: Vec<Credential> =
            self.credentials.read().await.values().cloned().collect();

        // Serialize to JSON
        let json = serde_json::to_string(&credentials)
            .map_err(|e| PasswordError::SerializationError(e.to_string()))?;

        // Generate export key
        let salt = SaltString::generate(&mut OsRng);
        let export_key = self.derive_key(export_password, salt.as_str())?;

        // Encrypt the JSON
        let encrypted = self.encrypt_string(&json, &export_key)?;

        // Create export structure
        let export_data = ExportData {
            version: 1,
            salt: salt.to_string(),
            data: encrypted,
        };

        serde_json::to_string(&export_data)
            .map_err(|e| PasswordError::SerializationError(e.to_string()))
    }

    /// Import credentials from encrypted export
    pub async fn import(&self, export_data: &str, export_password: &str) -> Result<usize> {
        if !self.is_unlocked().await {
            return Err(PasswordError::VaultLocked);
        }

        // Parse export structure
        let export: ExportData = serde_json::from_str(export_data)
            .map_err(|e| PasswordError::ImportExportError(e.to_string()))?;

        // Derive export key
        let export_key = self.derive_key(export_password, &export.salt)?;

        // Decrypt the data
        let json = self.decrypt_string(&export.data, &export_key)?;

        // Parse credentials
        let imported_credentials: Vec<Credential> = serde_json::from_str(&json)
            .map_err(|e| PasswordError::ImportExportError(e.to_string()))?;

        let count = imported_credentials.len();

        // Add to store (generate new IDs to avoid conflicts)
        let mut credentials = self.credentials.write().await;
        for mut cred in imported_credentials {
            cred.id = CredentialId::new();
            credentials.insert(cred.id, cred);
        }

        Ok(count)
    }

    /// Check if a password appears in known breaches
    ///
    /// Uses k-anonymity approach: only sends first 5 chars of SHA-1 hash
    /// In production, this would call the Have I Been Pwned API
    pub fn check_breach(&self, password: &str) -> BreachCheckResult {
        // Compute SHA-1 hash (HIBP uses SHA-1)
        use sha2::Sha256;
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let hash = format!("{:X}", hasher.finalize());

        // In production, would send hash[0..5] to HIBP API
        // For now, check against a small list of known compromised passwords
        let compromised = Self::is_known_compromised(password);

        if compromised {
            BreachCheckResult::Breached {
                count: Some(1),
                hash_prefix: hash[0..5].to_string(),
            }
        } else {
            BreachCheckResult::Safe {
                hash_prefix: hash[0..5].to_string(),
            }
        }
    }

    /// Calculate password strength score (0-100)
    pub fn calculate_strength(password: &str) -> PasswordStrength {
        let len = password.len();
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let mut score: u8 = 0;

        // Length scoring
        score += match len {
            0..=4 => 0,
            5..=7 => 10,
            8..=11 => 25,
            12..=15 => 40,
            16..=19 => 55,
            _ => 70,
        };

        // Character variety
        if has_upper {
            score += 10;
        }
        if has_lower {
            score += 5;
        }
        if has_digit {
            score += 10;
        }
        if has_special {
            score += 15;
        }

        // Penalize common patterns
        let password_lower = password.to_lowercase();
        if Self::is_common_password(&password_lower) {
            score = score.saturating_sub(50);
        }

        let score = score.min(100);

        let level = match score {
            0..=20 => StrengthLevel::VeryWeak,
            21..=40 => StrengthLevel::Weak,
            41..=60 => StrengthLevel::Fair,
            61..=80 => StrengthLevel::Strong,
            _ => StrengthLevel::VeryStrong,
        };

        PasswordStrength {
            score,
            level,
            has_uppercase: has_upper,
            has_lowercase: has_lower,
            has_digits: has_digit,
            has_special,
            length: len,
        }
    }

    // Private helper methods

    fn derive_key(&self, password: &str, salt: &str) -> Result<DerivedKey> {
        let salt = SaltString::from_b64(salt)
            .map_err(|e| PasswordError::KeyDerivationFailed(e.to_string()))?;

        let argon2 = Argon2::default();

        // Use password_hash to derive key
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PasswordError::KeyDerivationFailed(e.to_string()))?;

        // Extract 32 bytes from the hash
        let hash_str = hash.hash.ok_or_else(|| {
            PasswordError::KeyDerivationFailed("Failed to get hash output".to_string())
        })?;

        let hash_bytes = hash_str.as_bytes();
        let mut key = [0u8; 32];

        // Copy first 32 bytes or pad if necessary
        let copy_len = hash_bytes.len().min(32);
        key[..copy_len].copy_from_slice(&hash_bytes[..copy_len]);

        Ok(DerivedKey::new(key))
    }

    async fn get_key(&self) -> Result<DerivedKey> {
        self.derived_key
            .read()
            .await
            .clone()
            .ok_or(PasswordError::VaultLocked)
    }

    fn encrypt_string(&self, plaintext: &str, key: &DerivedKey) -> Result<EncryptedString> {
        let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
            .map_err(|e| PasswordError::EncryptionFailed(e.to_string()))?;

        // Generate random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| PasswordError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedString::new(ciphertext, nonce_bytes.to_vec()))
    }

    fn decrypt_string(&self, encrypted: &EncryptedString, key: &DerivedKey) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
            .map_err(|e| PasswordError::DecryptionFailed(e.to_string()))?;

        let nonce_bytes = encrypted.nonce_bytes()?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = encrypted.ciphertext_bytes()?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|e| PasswordError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext).map_err(|e| PasswordError::DecryptionFailed(e.to_string()))
    }

    fn extract_domain(url: &str) -> String {
        // Remove protocol
        let without_protocol = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);

        // Get domain part (before any path)
        let domain = without_protocol
            .split('/')
            .next()
            .unwrap_or(without_protocol);

        // Remove www prefix
        domain
            .strip_prefix("www.")
            .unwrap_or(domain)
            .to_string()
    }

    fn is_known_compromised(password: &str) -> bool {
        // Small list of commonly breached passwords for demo
        // In production, this would query HIBP API
        const COMPROMISED: &[&str] = &[
            "password",
            "123456",
            "123456789",
            "qwerty",
            "password123",
            "12345678",
            "111111",
            "1234567890",
            "password1",
            "qwerty123",
        ];

        COMPROMISED.contains(&password.to_lowercase().as_str())
    }

    fn is_common_password(password: &str) -> bool {
        Self::is_known_compromised(password)
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Export data structure
#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    version: u32,
    salt: String,
    data: EncryptedString,
}

/// Result of breach check
#[derive(Debug, Clone)]
pub enum BreachCheckResult {
    /// Password found in breach database
    Breached {
        /// Number of times seen (if known)
        count: Option<u64>,
        /// First 5 chars of hash (for reference)
        hash_prefix: String,
    },
    /// Password not found in breach database
    Safe {
        /// First 5 chars of hash (for reference)
        hash_prefix: String,
    },
}

impl BreachCheckResult {
    /// Check if the password was breached
    pub fn is_breached(&self) -> bool {
        matches!(self, BreachCheckResult::Breached { .. })
    }
}

/// Password strength level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrengthLevel {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

/// Password strength analysis
#[derive(Debug, Clone)]
pub struct PasswordStrength {
    /// Strength score (0-100)
    pub score: u8,
    /// Strength level
    pub level: StrengthLevel,
    /// Has uppercase letters
    pub has_uppercase: bool,
    /// Has lowercase letters
    pub has_lowercase: bool,
    /// Has digits
    pub has_digits: bool,
    /// Has special characters
    pub has_special: bool,
    /// Password length
    pub length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_id_uniqueness() {
        let id1 = CredentialId::new();
        let id2 = CredentialId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_password_generator_default() {
        let config = PasswordConfig::default();
        let password = PasswordGenerator::generate(&config).unwrap();

        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_password_generator_simple() {
        let config = PasswordConfig::simple(12);
        let password = PasswordGenerator::generate(&config).unwrap();

        assert_eq!(password.len(), 12);
        assert!(!password.chars().any(|c| "!@#$%^&*()".contains(c)));
    }

    #[test]
    fn test_password_generator_pin() {
        let config = PasswordConfig::pin(6);
        let password = PasswordGenerator::generate(&config).unwrap();

        assert_eq!(password.len(), 6);
        assert!(password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_password_generator_strong() {
        let config = PasswordConfig::strong(20);
        let password = PasswordGenerator::generate(&config).unwrap();

        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_password_generator_empty_charset() {
        let config = PasswordConfig {
            length: 10,
            uppercase: false,
            lowercase: false,
            digits: false,
            special: false,
            exclude_ambiguous: false,
            custom_chars: None,
            min_per_type: 0,
        };

        let result = PasswordGenerator::generate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_generator_zero_length() {
        let mut config = PasswordConfig::default();
        config.length = 0;

        let result = PasswordGenerator::generate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_strength_weak() {
        let strength = CredentialStore::calculate_strength("abc");
        assert_eq!(strength.level, StrengthLevel::VeryWeak);
    }

    #[test]
    fn test_password_strength_strong() {
        let strength = CredentialStore::calculate_strength("MyStr0ng!Password#2024");
        assert!(strength.score >= 60);
    }

    #[test]
    fn test_password_strength_common() {
        let strength = CredentialStore::calculate_strength("password");
        assert!(strength.score < 30);
    }

    #[test]
    fn test_breach_check_known() {
        let store = CredentialStore::new();
        let result = store.check_breach("password");
        assert!(result.is_breached());
    }

    #[test]
    fn test_breach_check_safe() {
        let store = CredentialStore::new();
        let result = store.check_breach("ThisIsAVeryUniquePa$$w0rd!");
        assert!(!result.is_breached());
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(
            CredentialStore::extract_domain("https://www.example.com/path"),
            "example.com"
        );
        assert_eq!(
            CredentialStore::extract_domain("http://example.com"),
            "example.com"
        );
        assert_eq!(
            CredentialStore::extract_domain("example.com"),
            "example.com"
        );
        assert_eq!(
            CredentialStore::extract_domain("https://sub.example.com"),
            "sub.example.com"
        );
    }

    #[tokio::test]
    async fn test_credential_store_lifecycle() {
        let store = CredentialStore::new();

        // Initialize with master password
        store.initialize("master_password").await.unwrap();
        assert!(store.is_unlocked().await);

        // Create a credential
        let id = store
            .create(
                "https://example.com".to_string(),
                "user@example.com".to_string(),
                "MySecretPassword123!",
                Some("My notes"),
            )
            .await
            .unwrap();

        // Get the credential
        let cred = store.get(id).await.unwrap();
        assert_eq!(cred.site, "https://example.com");
        assert_eq!(cred.username, "user@example.com");

        // Get decrypted credential
        let decrypted = store.get_decrypted(id).await.unwrap();
        assert_eq!(decrypted.password, "MySecretPassword123!");
        assert_eq!(decrypted.notes, Some("My notes".to_string()));

        // Lock the vault
        store.lock().await;
        assert!(!store.is_unlocked().await);

        // Try to get credential while locked
        let result = store.get(id).await;
        assert!(matches!(result, Err(PasswordError::VaultLocked)));

        // Unlock with correct password
        store.unlock("master_password").await.unwrap();
        assert!(store.is_unlocked().await);

        // Try to unlock with wrong password
        store.lock().await;
        let result = store.unlock("wrong_password").await;
        assert!(matches!(result, Err(PasswordError::InvalidMasterPassword)));
    }

    #[tokio::test]
    async fn test_find_for_site() {
        let store = CredentialStore::new();
        store.initialize("master").await.unwrap();

        // Create credentials for different sites
        store
            .create(
                "https://example.com".to_string(),
                "user1".to_string(),
                "pass1",
                None,
            )
            .await
            .unwrap();

        store
            .create(
                "https://www.example.com".to_string(),
                "user2".to_string(),
                "pass2",
                None,
            )
            .await
            .unwrap();

        store
            .create(
                "https://other.com".to_string(),
                "user3".to_string(),
                "pass3",
                None,
            )
            .await
            .unwrap();

        // Find credentials for example.com
        let matches = store.find_for_site("https://example.com/login").await;
        assert_eq!(matches.len(), 2);

        // Find credentials for other.com
        let matches = store.find_for_site("https://other.com").await;
        assert_eq!(matches.len(), 1);
    }

    #[tokio::test]
    async fn test_update_password() {
        let store = CredentialStore::new();
        store.initialize("master").await.unwrap();

        let id = store
            .create(
                "https://example.com".to_string(),
                "user".to_string(),
                "old_password",
                None,
            )
            .await
            .unwrap();

        // Update password
        store.update_password(id, "new_password").await.unwrap();

        // Verify update
        let decrypted = store.get_decrypted(id).await.unwrap();
        assert_eq!(decrypted.password, "new_password");
    }

    #[tokio::test]
    async fn test_delete_credential() {
        let store = CredentialStore::new();
        store.initialize("master").await.unwrap();

        let id = store
            .create(
                "https://example.com".to_string(),
                "user".to_string(),
                "password",
                None,
            )
            .await
            .unwrap();

        // Delete
        store.delete(id).await.unwrap();

        // Verify deleted
        let result = store.get(id).await;
        assert!(matches!(result, Err(PasswordError::CredentialNotFound(_))));
    }

    #[tokio::test]
    async fn test_export_import() {
        let store1 = CredentialStore::new();
        store1.initialize("master1").await.unwrap();

        // Create some credentials
        store1
            .create(
                "https://example.com".to_string(),
                "user1".to_string(),
                "pass1",
                Some("notes1"),
            )
            .await
            .unwrap();

        store1
            .create(
                "https://other.com".to_string(),
                "user2".to_string(),
                "pass2",
                None,
            )
            .await
            .unwrap();

        // Export
        let exported = store1.export("export_password").await.unwrap();

        // Import into new store
        let store2 = CredentialStore::new();
        store2.initialize("master2").await.unwrap();

        let count = store2.import(&exported, "export_password").await.unwrap();
        assert_eq!(count, 2);

        // Verify imported credentials
        let credentials = store2.list().await.unwrap();
        assert_eq!(credentials.len(), 2);
    }
}
