//! SettingsManager implementation
//!
//! Manages user settings with persistence to disk using YAML format

use crate::{defaults, SettingValue};
use shared_types::ComponentError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages user settings and preferences
#[derive(Clone)]
pub struct SettingsManager {
    /// Internal settings storage (thread-safe)
    settings: Arc<RwLock<HashMap<String, SettingValue>>>,
    /// Configuration directory path
    config_dir: Arc<PathBuf>,
}

impl SettingsManager {
    /// Create a new SettingsManager with defaults loaded
    pub fn new() -> Self {
        let config_dir = Self::default_config_dir();
        Self {
            settings: Arc::new(RwLock::new(defaults::create_defaults())),
            config_dir: Arc::new(config_dir),
        }
    }

    /// Create a SettingsManager with a specific config directory
    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        Self {
            settings: Arc::new(RwLock::new(defaults::create_defaults())),
            config_dir: Arc::new(config_dir),
        }
    }

    /// Get the default configuration directory
    fn default_config_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "corten", "browser-shell")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Get a setting value by key
    pub async fn get_setting(&self, key: &str) -> Result<SettingValue, ComponentError> {
        let settings = self.settings.read().await;
        settings
            .get(key)
            .cloned()
            .ok_or_else(|| ComponentError::ResourceNotFound(format!("Setting not found: {}", key)))
    }

    /// Set a setting value
    pub async fn set_setting(
        &self,
        key: String,
        value: SettingValue,
    ) -> Result<(), ComponentError> {
        let mut settings = self.settings.write().await;
        settings.insert(key, value);
        Ok(())
    }

    /// Get all settings as a HashMap
    pub async fn get_all_settings(&self) -> Result<HashMap<String, SettingValue>, ComponentError> {
        let settings = self.settings.read().await;
        Ok(settings.clone())
    }

    /// Reset all settings to defaults
    pub async fn reset_to_defaults(&self) -> Result<(), ComponentError> {
        let mut settings = self.settings.write().await;
        *settings = defaults::create_defaults();
        Ok(())
    }

    /// Save settings to disk
    pub async fn save(&self) -> Result<(), ComponentError> {
        let settings = self.settings.read().await;
        let config_file = self.config_dir.join("settings.yaml");

        // Create config directory if it doesn't exist
        if let Some(parent) = config_file.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ComponentError::InitializationFailed(format!(
                    "Failed to create config directory: {}",
                    e
                ))
            })?;
        }

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&*settings).map_err(|e| {
            ComponentError::InvalidState(format!("Failed to serialize settings: {}", e))
        })?;

        // Write to file
        tokio::fs::write(&config_file, yaml).await.map_err(|e| {
            ComponentError::InitializationFailed(format!("Failed to write settings file: {}", e))
        })?;

        Ok(())
    }

    /// Load settings from disk
    pub async fn load(&self) -> Result<(), ComponentError> {
        let config_file = self.config_dir.join("settings.yaml");

        // If file doesn't exist, just use defaults (already loaded)
        if !config_file.exists() {
            return Ok(());
        }

        // Read file
        let yaml = tokio::fs::read_to_string(&config_file).await.map_err(|e| {
            ComponentError::InitializationFailed(format!("Failed to read settings file: {}", e))
        })?;

        // Parse YAML
        let loaded_settings: HashMap<String, SettingValue> =
            serde_yaml::from_str(&yaml).map_err(|e| {
                ComponentError::InvalidState(format!("Failed to parse settings file: {}", e))
            })?;

        // Merge with defaults (loaded settings override defaults)
        let mut settings = self.settings.write().await;
        let defaults = defaults::create_defaults();

        // Start with defaults
        *settings = defaults;

        // Override with loaded settings
        for (key, value) in loaded_settings {
            settings.insert(key, value);
        }

        Ok(())
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_has_defaults() {
        let manager = SettingsManager::new();
        let all = manager.get_all_settings().await.unwrap();
        assert!(!all.is_empty());
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let manager = SettingsManager::new();
        manager
            .set_setting(
                "test.key".to_string(),
                SettingValue::String("value".to_string()),
            )
            .await
            .unwrap();
        let value = manager.get_setting("test.key").await.unwrap();
        match value {
            SettingValue::String(s) => assert_eq!(s, "value"),
            _ => panic!("Expected String"),
        }
    }
}
