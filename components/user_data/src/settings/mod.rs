// @implements: REQ-007
//! Settings Manager Module
//!
//! Provides key-value settings storage using SQLite as the backend.
//! Settings persist across browser restarts.

mod storage;

use rusqlite::Connection;
use std::collections::HashMap;
pub use storage::SettingsStorage;

/// Error types for settings operations
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

pub type SettingsResult<T> = Result<T, SettingsError>;

/// Settings Manager - provides key-value settings storage
pub struct SettingsManager {
    storage: SettingsStorage,
}

impl SettingsManager {
    /// Create a new SettingsManager with the given database connection
    pub fn new(conn: Connection) -> SettingsResult<Self> {
        let storage = SettingsStorage::new(conn)?;
        Ok(Self { storage })
    }

    /// Set a setting value
    pub fn set(&mut self, key: &str, value: &str) -> SettingsResult<()> {
        if key.is_empty() {
            return Err(SettingsError::InvalidKey("Key cannot be empty".to_string()));
        }
        self.storage.set(key, value)
    }

    /// Get a setting value
    pub fn get(&self, key: &str) -> SettingsResult<Option<String>> {
        self.storage.get(key)
    }

    /// Delete a setting
    /// Returns true if the setting existed and was deleted, false otherwise
    pub fn delete(&mut self, key: &str) -> SettingsResult<bool> {
        self.storage.delete(key)
    }

    /// List all settings as a HashMap
    pub fn list_all(&self) -> SettingsResult<HashMap<String, String>> {
        self.storage.list_all()
    }

    /// Clear all settings
    pub fn clear_all(&mut self) -> SettingsResult<()> {
        self.storage.clear_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> SettingsManager {
        let conn = Connection::open_in_memory().unwrap();
        SettingsManager::new(conn).unwrap()
    }

    #[test]
    fn test_set_empty_key_returns_error() {
        let mut manager = create_test_manager();
        let result = manager.set("", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_set_get() {
        let mut manager = create_test_manager();
        manager.set("test_key", "test_value").unwrap();
        let value = manager.get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }
}
