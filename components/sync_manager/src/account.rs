//! Sync account management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a sync account for cross-device synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAccount {
    /// Unique account identifier
    pub id: String,

    /// User's email address
    pub email: String,

    /// Display name
    pub display_name: Option<String>,

    /// When the account was created
    pub created_at: DateTime<Utc>,

    /// When the user last authenticated
    pub last_authenticated: DateTime<Utc>,

    /// Whether the account is verified
    pub is_verified: bool,

    /// Current sync server endpoint
    pub sync_server: String,

    /// Device-specific account settings
    pub device_settings: DeviceSettings,
}

/// Device-specific settings for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSettings {
    /// Unique device identifier
    pub device_id: String,

    /// Human-readable device name
    pub device_name: String,

    /// When this device was registered
    pub registered_at: DateTime<Utc>,

    /// Data types enabled for sync on this device
    pub enabled_types: Vec<crate::SyncDataType>,

    /// Whether to sync over metered connections
    pub sync_on_metered: bool,

    /// Maximum sync frequency in seconds (0 = realtime)
    pub sync_interval_seconds: u64,
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            device_id: uuid::Uuid::new_v4().to_string(),
            device_name: get_default_device_name(),
            registered_at: Utc::now(),
            enabled_types: crate::SyncDataType::all(),
            sync_on_metered: false,
            sync_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Credentials for sync account authentication
#[derive(Debug, Clone)]
pub struct SyncAccountCredentials {
    /// User's email address
    pub email: String,

    /// Authentication token (JWT or similar)
    pub auth_token: String,

    /// Refresh token for renewing auth
    pub refresh_token: Option<String>,

    /// When the auth token expires
    pub expires_at: Option<DateTime<Utc>>,
}

impl SyncAccountCredentials {
    /// Create new credentials
    pub fn new(email: String, auth_token: String) -> Self {
        Self {
            email,
            auth_token,
            refresh_token: None,
            expires_at: None,
        }
    }

    /// Check if credentials are expired
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| exp < Utc::now())
            .unwrap_or(false)
    }

    /// Check if credentials need refresh (within 5 minutes of expiry)
    pub fn needs_refresh(&self) -> bool {
        self.expires_at
            .map(|exp| exp < Utc::now() + chrono::Duration::minutes(5))
            .unwrap_or(false)
    }
}

impl SyncAccount {
    /// Create a new sync account
    pub fn new(id: String, email: String, sync_server: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            display_name: None,
            created_at: now,
            last_authenticated: now,
            is_verified: false,
            sync_server,
            device_settings: DeviceSettings::default(),
        }
    }

    /// Update last authenticated timestamp
    pub fn update_authenticated(&mut self) {
        self.last_authenticated = Utc::now();
    }

    /// Check if a specific data type is enabled for sync
    pub fn is_type_enabled(&self, data_type: crate::SyncDataType) -> bool {
        self.device_settings.enabled_types.contains(&data_type)
    }

    /// Enable a data type for sync
    pub fn enable_type(&mut self, data_type: crate::SyncDataType) {
        if !self.is_type_enabled(data_type) {
            self.device_settings.enabled_types.push(data_type);
        }
    }

    /// Disable a data type for sync
    pub fn disable_type(&mut self, data_type: crate::SyncDataType) {
        self.device_settings.enabled_types.retain(|t| *t != data_type);
    }
}

/// Get a default device name based on system information
fn get_default_device_name() -> String {
    // Try to get hostname from environment or use generic name
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "Unknown Device".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SyncDataType;

    #[test]
    fn test_sync_account_creation() {
        let account = SyncAccount::new(
            "acc_123".to_string(),
            "user@example.com".to_string(),
            "https://sync.example.com".to_string(),
        );

        assert_eq!(account.id, "acc_123");
        assert_eq!(account.email, "user@example.com");
        assert!(!account.is_verified);
    }

    #[test]
    fn test_type_enabled_management() {
        let mut account = SyncAccount::new(
            "acc_1".to_string(),
            "test@example.com".to_string(),
            "https://sync.example.com".to_string(),
        );

        // All types should be enabled by default
        assert!(account.is_type_enabled(SyncDataType::Bookmarks));
        assert!(account.is_type_enabled(SyncDataType::Passwords));

        // Disable and verify
        account.disable_type(SyncDataType::Passwords);
        assert!(!account.is_type_enabled(SyncDataType::Passwords));
        assert!(account.is_type_enabled(SyncDataType::Bookmarks));

        // Re-enable
        account.enable_type(SyncDataType::Passwords);
        assert!(account.is_type_enabled(SyncDataType::Passwords));
    }

    #[test]
    fn test_credentials_expiry() {
        let mut creds = SyncAccountCredentials::new(
            "test@example.com".to_string(),
            "token_123".to_string(),
        );

        // No expiry set
        assert!(!creds.is_expired());
        assert!(!creds.needs_refresh());

        // Set expiry in the past
        creds.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(creds.is_expired());
        assert!(creds.needs_refresh());

        // Set expiry in the future (but within refresh window)
        creds.expires_at = Some(Utc::now() + chrono::Duration::minutes(3));
        assert!(!creds.is_expired());
        assert!(creds.needs_refresh());

        // Set expiry far in the future
        creds.expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(!creds.is_expired());
        assert!(!creds.needs_refresh());
    }

    #[test]
    fn test_device_settings_default() {
        let settings = DeviceSettings::default();

        assert!(!settings.device_id.is_empty());
        assert!(!settings.device_name.is_empty());
        assert_eq!(settings.enabled_types.len(), 5);
        assert!(!settings.sync_on_metered);
        assert_eq!(settings.sync_interval_seconds, 300);
    }
}
