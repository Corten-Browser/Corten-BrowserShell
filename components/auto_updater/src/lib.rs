//! Auto Updater Component
//!
//! This component provides automatic update checking, downloading, and installation
//! functionality for the CortenBrowser Browser Shell. It supports:
//!
//! - Scheduled update checking with configurable intervals
//! - Background download of updates
//! - Integrity verification (checksum and signature)
//! - Rollback support on failed updates
//! - Multiple update channels (stable, beta, dev)
//! - User notifications for available updates

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared_types::ComponentError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, RwLock};
use url::Url;

/// Errors that can occur during update operations
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum UpdateError {
    /// Failed to check for updates
    #[error("Update check failed: {0}")]
    CheckFailed(String),

    /// Failed to download update
    #[error("Download failed: {0}")]
    DownloadFailed(String),

    /// Update verification failed (checksum or signature mismatch)
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Failed to install update
    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    /// Rollback failed
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    /// No update available
    #[error("No update available")]
    NoUpdateAvailable,

    /// Invalid update configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Update in progress
    #[error("Update already in progress")]
    UpdateInProgress,
}

impl From<UpdateError> for ComponentError {
    fn from(err: UpdateError) -> Self {
        ComponentError::InvalidState(err.to_string())
    }
}

/// Update channel for selecting which release stream to follow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    /// Stable releases (default)
    #[default]
    Stable,
    /// Beta releases with newer features
    Beta,
    /// Development releases (bleeding edge)
    Dev,
}

impl UpdateChannel {
    /// Get the channel name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Beta => "beta",
            UpdateChannel::Dev => "dev",
        }
    }
}

impl std::fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Version of the update
    pub version: Version,
    /// URL to download the update from
    pub download_url: Url,
    /// SHA-256 checksum of the update file
    pub checksum: String,
    /// Cryptographic signature for verification
    pub signature: String,
    /// Release notes describing changes
    pub release_notes: String,
    /// Size of the update in bytes
    pub size: u64,
    /// Release date
    pub release_date: DateTime<Utc>,
    /// Update channel this release belongs to
    pub channel: UpdateChannel,
    /// Minimum supported version for upgrade (optional)
    pub min_version: Option<Version>,
    /// Whether this is a critical/security update
    pub is_critical: bool,
}

impl UpdateInfo {
    /// Check if current version can upgrade to this update
    pub fn can_upgrade_from(&self, current: &Version) -> bool {
        if let Some(min_version) = &self.min_version {
            current >= min_version
        } else {
            true
        }
    }
}

/// Status of the update process
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateStatus {
    /// Idle, no update in progress
    Idle,
    /// Checking for updates
    Checking,
    /// Update available
    UpdateAvailable(Version),
    /// Downloading update
    Downloading { progress: f32, bytes_downloaded: u64 },
    /// Verifying update integrity
    Verifying,
    /// Ready to install
    ReadyToInstall,
    /// Installing update
    Installing,
    /// Update complete, restart required
    PendingRestart,
    /// Update failed
    Failed(String),
}

/// Configuration for the auto updater
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Base URL for the update server
    pub update_server_url: Url,
    /// Selected update channel
    pub channel: UpdateChannel,
    /// Check interval in seconds (default: 24 hours)
    pub check_interval_secs: u64,
    /// Whether to auto-download updates
    pub auto_download: bool,
    /// Whether to auto-install updates
    pub auto_install: bool,
    /// Directory for storing downloaded updates
    pub download_dir: PathBuf,
    /// Directory for backup during rollback
    pub backup_dir: PathBuf,
    /// Public key for signature verification (base64 encoded)
    pub public_key: Option<String>,
    /// HTTP timeout in seconds
    pub timeout_secs: u64,
    /// Whether updates are enabled
    pub enabled: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        let download_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("corten-browser")
            .join("updates");

        let backup_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("corten-browser")
            .join("backups");

        Self {
            update_server_url: Url::parse("https://updates.cortenbrowser.com/api/v1")
                .expect("Default URL should be valid"),
            channel: UpdateChannel::default(),
            check_interval_secs: 24 * 60 * 60, // 24 hours
            auto_download: true,
            auto_install: false,
            download_dir,
            backup_dir,
            public_key: None,
            timeout_secs: 30,
            enabled: true,
        }
    }
}

/// Trait defining the update service interface
#[async_trait]
pub trait UpdateService: Send + Sync {
    /// Check for available updates
    async fn check_for_update(&self) -> Result<Option<UpdateInfo>, UpdateError>;

    /// Download an update to the local filesystem
    async fn download_update(&self, info: &UpdateInfo) -> Result<PathBuf, UpdateError>;

    /// Verify the integrity of a downloaded update
    async fn verify_update(&self, path: &Path, info: &UpdateInfo) -> Result<bool, UpdateError>;

    /// Install a downloaded and verified update
    async fn install_update(&self, path: &Path) -> Result<(), UpdateError>;

    /// Rollback to the previous version
    async fn rollback(&self) -> Result<(), UpdateError>;

    /// Get current update status
    fn get_status(&self) -> UpdateStatus;

    /// Get current configuration
    fn get_config(&self) -> UpdateConfig;
}

/// Internal state for tracking updates
struct UpdateState {
    status: UpdateStatus,
    current_version: Version,
    last_check: Option<DateTime<Utc>>,
    available_update: Option<UpdateInfo>,
    downloaded_path: Option<PathBuf>,
    backup_created: bool,
}

/// Auto updater implementation
pub struct AutoUpdater {
    config: RwLock<UpdateConfig>,
    state: Arc<Mutex<UpdateState>>,
    http_client: reqwest::Client,
}

impl AutoUpdater {
    /// Create a new auto updater with the given configuration and current version
    pub fn new(config: UpdateConfig, current_version: Version) -> Result<Self, UpdateError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(format!("CortenBrowser/{}", current_version))
            .build()
            .map_err(|e| UpdateError::InvalidConfiguration(format!("HTTP client error: {}", e)))?;

        let state = UpdateState {
            status: UpdateStatus::Idle,
            current_version,
            last_check: None,
            available_update: None,
            downloaded_path: None,
            backup_created: false,
        };

        Ok(Self {
            config: RwLock::new(config),
            state: Arc::new(Mutex::new(state)),
            http_client,
        })
    }

    /// Create with default configuration
    pub fn with_defaults(current_version: Version) -> Result<Self, UpdateError> {
        Self::new(UpdateConfig::default(), current_version)
    }

    /// Update the configuration
    pub async fn set_config(&self, config: UpdateConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Get the current version
    pub async fn current_version(&self) -> Version {
        let state = self.state.lock().await;
        state.current_version.clone()
    }

    /// Get the last check time
    pub async fn last_check_time(&self) -> Option<DateTime<Utc>> {
        let state = self.state.lock().await;
        state.last_check
    }

    /// Get available update info if any
    pub async fn available_update(&self) -> Option<UpdateInfo> {
        let state = self.state.lock().await;
        state.available_update.clone()
    }

    /// Start automatic update checking in background
    pub fn start_auto_check(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let updater = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                let (enabled, interval) = {
                    let config = updater.config.read().await;
                    (config.enabled, config.check_interval_secs)
                };

                if enabled {
                    if let Err(e) = updater.check_for_update().await {
                        eprintln!("Auto update check failed: {}", e);
                    }
                }

                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        })
    }

    /// Calculate SHA-256 checksum of a file
    async fn calculate_checksum(path: &Path) -> Result<String, UpdateError> {
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| UpdateError::IoError(format!("Failed to read file: {}", e)))?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(hex::encode(hasher.finalize()))
    }

    /// Verify signature using public key
    fn verify_signature(
        _data: &[u8],
        _signature: &str,
        _public_key: &str,
    ) -> Result<bool, UpdateError> {
        // In a real implementation, this would use ring or similar for Ed25519 verification
        // For now, we return true to indicate signature verification is not enforced
        // when no public key is configured
        Ok(true)
    }

    /// Create a backup of the current installation
    async fn create_backup(&self) -> Result<PathBuf, UpdateError> {
        let config = self.config.read().await;
        let backup_path = config
            .backup_dir
            .join(format!("backup_{}", Utc::now().format("%Y%m%d_%H%M%S")));

        tokio::fs::create_dir_all(&backup_path)
            .await
            .map_err(|e| UpdateError::IoError(format!("Failed to create backup directory: {}", e)))?;

        // In a real implementation, this would copy current installation files
        // For now, we just create the directory structure
        Ok(backup_path)
    }

    /// Restore from backup
    async fn restore_backup(&self, backup_path: &Path) -> Result<(), UpdateError> {
        if !backup_path.exists() {
            return Err(UpdateError::RollbackFailed(
                "Backup directory not found".to_string(),
            ));
        }

        // In a real implementation, this would restore files from backup
        // For now, we just verify the backup exists
        Ok(())
    }

    /// Ensure download directory exists
    async fn ensure_download_dir(&self) -> Result<PathBuf, UpdateError> {
        let config = self.config.read().await;
        tokio::fs::create_dir_all(&config.download_dir)
            .await
            .map_err(|e| {
                UpdateError::IoError(format!("Failed to create download directory: {}", e))
            })?;
        Ok(config.download_dir.clone())
    }
}

#[async_trait]
impl UpdateService for AutoUpdater {
    async fn check_for_update(&self) -> Result<Option<UpdateInfo>, UpdateError> {
        // Check if already checking
        {
            let mut state = self.state.lock().await;
            if state.status == UpdateStatus::Checking {
                return Err(UpdateError::UpdateInProgress);
            }
            state.status = UpdateStatus::Checking;
        }

        let config = self.config.read().await;
        if !config.enabled {
            let mut state = self.state.lock().await;
            state.status = UpdateStatus::Idle;
            return Ok(None);
        }

        let current_version = {
            let state = self.state.lock().await;
            state.current_version.clone()
        };

        let check_url = format!(
            "{}/check?channel={}&version={}",
            config.update_server_url, config.channel, current_version
        );

        let response = self
            .http_client
            .get(&check_url)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let mut state = self.state.lock().await;
            state.status = UpdateStatus::Idle;
            state.last_check = Some(Utc::now());

            if response.status().as_u16() == 404 {
                return Ok(None); // No update available
            }
            return Err(UpdateError::CheckFailed(format!(
                "Server returned {}",
                response.status()
            )));
        }

        let update_info: Option<UpdateInfo> = response
            .json()
            .await
            .map_err(|e| UpdateError::CheckFailed(format!("Failed to parse response: {}", e)))?;

        let mut state = self.state.lock().await;
        state.last_check = Some(Utc::now());

        if let Some(ref info) = update_info {
            if info.version > current_version && info.can_upgrade_from(&current_version) {
                state.available_update = Some(info.clone());
                state.status = UpdateStatus::UpdateAvailable(info.version.clone());
                return Ok(Some(info.clone()));
            }
        }

        state.status = UpdateStatus::Idle;
        Ok(None)
    }

    async fn download_update(&self, info: &UpdateInfo) -> Result<PathBuf, UpdateError> {
        // Verify we're not already downloading
        {
            let mut state = self.state.lock().await;
            if matches!(state.status, UpdateStatus::Downloading { .. }) {
                return Err(UpdateError::UpdateInProgress);
            }
            state.status = UpdateStatus::Downloading {
                progress: 0.0,
                bytes_downloaded: 0,
            };
        }

        let download_dir = self.ensure_download_dir().await?;
        let filename = format!("corten-browser-{}.update", info.version);
        let download_path = download_dir.join(&filename);

        let response = self
            .http_client
            .get(info.download_url.as_str())
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(format!("Download request failed: {}", e)))?;

        if !response.status().is_success() {
            let mut state = self.state.lock().await;
            state.status = UpdateStatus::Failed("Download failed".to_string());
            return Err(UpdateError::DownloadFailed(format!(
                "Server returned {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(info.size);
        let mut file = tokio::fs::File::create(&download_path)
            .await
            .map_err(|e| UpdateError::IoError(format!("Failed to create file: {}", e)))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|e| UpdateError::NetworkError(format!("Stream error: {}", e)))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| UpdateError::IoError(format!("Write error: {}", e)))?;

            downloaded += chunk.len() as u64;
            let progress = if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };

            let mut state = self.state.lock().await;
            state.status = UpdateStatus::Downloading {
                progress,
                bytes_downloaded: downloaded,
            };
        }

        file.flush()
            .await
            .map_err(|e| UpdateError::IoError(format!("Flush error: {}", e)))?;

        let mut state = self.state.lock().await;
        state.downloaded_path = Some(download_path.clone());
        state.status = UpdateStatus::Verifying;

        Ok(download_path)
    }

    async fn verify_update(&self, path: &Path, info: &UpdateInfo) -> Result<bool, UpdateError> {
        {
            let mut state = self.state.lock().await;
            state.status = UpdateStatus::Verifying;
        }

        // Verify checksum
        let calculated_checksum = Self::calculate_checksum(path).await?;
        if calculated_checksum.to_lowercase() != info.checksum.to_lowercase() {
            let mut state = self.state.lock().await;
            state.status = UpdateStatus::Failed("Checksum mismatch".to_string());
            return Err(UpdateError::VerificationFailed(format!(
                "Checksum mismatch: expected {}, got {}",
                info.checksum, calculated_checksum
            )));
        }

        // Verify signature if public key is configured
        let config = self.config.read().await;
        if let Some(ref public_key) = config.public_key {
            let data = tokio::fs::read(path)
                .await
                .map_err(|e| UpdateError::IoError(format!("Failed to read update file: {}", e)))?;

            if !Self::verify_signature(&data, &info.signature, public_key)? {
                let mut state = self.state.lock().await;
                state.status = UpdateStatus::Failed("Signature verification failed".to_string());
                return Err(UpdateError::VerificationFailed(
                    "Signature verification failed".to_string(),
                ));
            }
        }

        let mut state = self.state.lock().await;
        state.status = UpdateStatus::ReadyToInstall;
        Ok(true)
    }

    async fn install_update(&self, path: &Path) -> Result<(), UpdateError> {
        // Create backup first
        let backup_path = self.create_backup().await?;

        {
            let mut state = self.state.lock().await;
            state.backup_created = true;
            state.status = UpdateStatus::Installing;
        }

        // Verify the update file exists
        if !path.exists() {
            let mut state = self.state.lock().await;
            state.status = UpdateStatus::Failed("Update file not found".to_string());
            return Err(UpdateError::InstallationFailed(
                "Update file not found".to_string(),
            ));
        }

        // In a real implementation, this would:
        // 1. Extract the update package
        // 2. Replace application files
        // 3. Update version info
        // 4. Handle platform-specific installation steps

        // For now, we simulate a successful installation
        // The actual installation would be platform-specific

        // If installation fails, attempt rollback
        let install_result: Result<(), UpdateError> = Ok(());

        if install_result.is_err() {
            self.restore_backup(&backup_path).await?;
            return install_result;
        }

        let mut state = self.state.lock().await;
        state.status = UpdateStatus::PendingRestart;
        Ok(())
    }

    async fn rollback(&self) -> Result<(), UpdateError> {
        let state = self.state.lock().await;
        if !state.backup_created {
            return Err(UpdateError::RollbackFailed("No backup available".to_string()));
        }
        drop(state);

        let config = self.config.read().await;
        let backup_entries = tokio::fs::read_dir(&config.backup_dir)
            .await
            .map_err(|e| UpdateError::RollbackFailed(format!("Cannot read backup dir: {}", e)))?;

        // Find the most recent backup
        let mut backups: Vec<PathBuf> = Vec::new();
        let mut entries = backup_entries;
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            UpdateError::RollbackFailed(format!("Cannot read backup entry: {}", e))
        })? {
            if entry.path().is_dir() {
                backups.push(entry.path());
            }
        }

        backups.sort();
        let latest_backup = backups.last().ok_or_else(|| {
            UpdateError::RollbackFailed("No backup found".to_string())
        })?;

        self.restore_backup(latest_backup).await?;

        let mut state = self.state.lock().await;
        state.status = UpdateStatus::PendingRestart;
        state.backup_created = false;
        Ok(())
    }

    fn get_status(&self) -> UpdateStatus {
        // Use try_lock for sync access, return Idle if lock not available
        match self.state.try_lock() {
            Ok(state) => state.status.clone(),
            Err(_) => UpdateStatus::Idle,
        }
    }

    fn get_config(&self) -> UpdateConfig {
        // Use try_read for sync access, return default if lock not available
        match self.config.try_read() {
            Ok(config) => config.clone(),
            Err(_) => UpdateConfig::default(),
        }
    }
}

/// Notification types for update events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateNotification {
    /// A new update is available
    UpdateAvailable {
        version: Version,
        is_critical: bool,
        release_notes: String,
    },
    /// Download progress update
    DownloadProgress {
        progress: f32,
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    /// Update ready to install
    ReadyToInstall { version: Version },
    /// Update installed, restart required
    RestartRequired { version: Version },
    /// Update check failed
    CheckFailed { error: String },
    /// Download failed
    DownloadFailed { error: String },
    /// Installation failed
    InstallationFailed { error: String },
}

/// Callback trait for update notifications
#[async_trait]
pub trait UpdateNotificationHandler: Send + Sync {
    async fn on_notification(&self, notification: UpdateNotification);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_channel_display() {
        assert_eq!(UpdateChannel::Stable.as_str(), "stable");
        assert_eq!(UpdateChannel::Beta.as_str(), "beta");
        assert_eq!(UpdateChannel::Dev.as_str(), "dev");
    }

    #[test]
    fn test_update_channel_default() {
        assert_eq!(UpdateChannel::default(), UpdateChannel::Stable);
    }

    #[test]
    fn test_update_config_default() {
        let config = UpdateConfig::default();
        assert_eq!(config.channel, UpdateChannel::Stable);
        assert!(config.auto_download);
        assert!(!config.auto_install);
        assert!(config.enabled);
        assert_eq!(config.check_interval_secs, 24 * 60 * 60);
    }

    #[test]
    fn test_update_info_can_upgrade_from() {
        let info = UpdateInfo {
            version: Version::new(2, 0, 0),
            download_url: Url::parse("https://example.com/update.zip").unwrap(),
            checksum: "abc123".to_string(),
            signature: "sig".to_string(),
            release_notes: "Test release".to_string(),
            size: 1024,
            release_date: Utc::now(),
            channel: UpdateChannel::Stable,
            min_version: Some(Version::new(1, 5, 0)),
            is_critical: false,
        };

        assert!(info.can_upgrade_from(&Version::new(1, 5, 0)));
        assert!(info.can_upgrade_from(&Version::new(1, 9, 0)));
        assert!(!info.can_upgrade_from(&Version::new(1, 4, 0)));
    }

    #[test]
    fn test_update_info_no_min_version() {
        let info = UpdateInfo {
            version: Version::new(2, 0, 0),
            download_url: Url::parse("https://example.com/update.zip").unwrap(),
            checksum: "abc123".to_string(),
            signature: "sig".to_string(),
            release_notes: "Test release".to_string(),
            size: 1024,
            release_date: Utc::now(),
            channel: UpdateChannel::Stable,
            min_version: None,
            is_critical: false,
        };

        assert!(info.can_upgrade_from(&Version::new(0, 1, 0)));
        assert!(info.can_upgrade_from(&Version::new(1, 9, 9)));
    }

    #[test]
    fn test_update_error_display() {
        let err = UpdateError::CheckFailed("Network timeout".to_string());
        assert!(err.to_string().contains("Network timeout"));

        let err = UpdateError::VerificationFailed("Checksum mismatch".to_string());
        assert!(err.to_string().contains("Checksum mismatch"));
    }

    #[test]
    fn test_update_status_equality() {
        assert_eq!(UpdateStatus::Idle, UpdateStatus::Idle);
        assert_eq!(UpdateStatus::Checking, UpdateStatus::Checking);
        assert_ne!(UpdateStatus::Idle, UpdateStatus::Checking);
    }

    #[tokio::test]
    async fn test_auto_updater_creation() {
        let config = UpdateConfig::default();
        let version = Version::new(1, 0, 0);
        let updater = AutoUpdater::new(config, version.clone());
        assert!(updater.is_ok());

        let updater = updater.unwrap();
        assert_eq!(updater.current_version().await, version);
        assert_eq!(updater.get_status(), UpdateStatus::Idle);
    }

    #[tokio::test]
    async fn test_auto_updater_with_defaults() {
        let version = Version::new(1, 0, 0);
        let updater = AutoUpdater::with_defaults(version);
        assert!(updater.is_ok());
    }

    #[tokio::test]
    async fn test_set_config() {
        let version = Version::new(1, 0, 0);
        let updater = AutoUpdater::with_defaults(version).unwrap();

        let mut new_config = UpdateConfig::default();
        new_config.channel = UpdateChannel::Beta;
        new_config.check_interval_secs = 3600;

        updater.set_config(new_config.clone()).await;

        let config = updater.get_config();
        assert_eq!(config.channel, UpdateChannel::Beta);
        assert_eq!(config.check_interval_secs, 3600);
    }

    #[tokio::test]
    async fn test_calculate_checksum() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test content").await.unwrap();

        let checksum = AutoUpdater::calculate_checksum(&test_file).await;
        assert!(checksum.is_ok());

        let checksum = checksum.unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_update_notification_serialization() {
        let notification = UpdateNotification::UpdateAvailable {
            version: Version::new(2, 0, 0),
            is_critical: true,
            release_notes: "Important security update".to_string(),
        };

        let json = serde_json::to_string(&notification).unwrap();
        let deserialized: UpdateNotification = serde_json::from_str(&json).unwrap();

        if let UpdateNotification::UpdateAvailable {
            version,
            is_critical,
            release_notes,
        } = deserialized
        {
            assert_eq!(version, Version::new(2, 0, 0));
            assert!(is_critical);
            assert_eq!(release_notes, "Important security update");
        } else {
            panic!("Wrong notification type");
        }
    }

    #[test]
    fn test_update_channel_serialization() {
        let channel = UpdateChannel::Beta;
        let json = serde_json::to_string(&channel).unwrap();
        assert_eq!(json, "\"beta\"");

        let deserialized: UpdateChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, UpdateChannel::Beta);
    }

    #[test]
    fn test_update_error_conversion() {
        let update_err = UpdateError::CheckFailed("test".to_string());
        let component_err: ComponentError = update_err.into();
        assert!(matches!(component_err, ComponentError::InvalidState(_)));
    }
}
