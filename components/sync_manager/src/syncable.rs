//! Syncable data types and trait definitions

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Change, SyncResult};

/// Types of data that can be synchronized across devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDataType {
    /// Browser bookmarks
    Bookmarks,
    /// Browsing history
    History,
    /// User settings and preferences
    Settings,
    /// Saved passwords and credentials
    Passwords,
    /// Currently open tabs
    OpenTabs,
}

impl fmt::Display for SyncDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncDataType::Bookmarks => write!(f, "bookmarks"),
            SyncDataType::History => write!(f, "history"),
            SyncDataType::Settings => write!(f, "settings"),
            SyncDataType::Passwords => write!(f, "passwords"),
            SyncDataType::OpenTabs => write!(f, "open_tabs"),
        }
    }
}

impl SyncDataType {
    /// Get all sync data types
    pub fn all() -> Vec<SyncDataType> {
        vec![
            SyncDataType::Bookmarks,
            SyncDataType::History,
            SyncDataType::Settings,
            SyncDataType::Passwords,
            SyncDataType::OpenTabs,
        ]
    }

    /// Check if this data type requires encryption
    pub fn requires_encryption(&self) -> bool {
        matches!(self, SyncDataType::Passwords | SyncDataType::Settings)
    }

    /// Get the sync priority (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            SyncDataType::Settings => 1,      // Highest priority
            SyncDataType::Passwords => 2,
            SyncDataType::Bookmarks => 3,
            SyncDataType::OpenTabs => 4,
            SyncDataType::History => 5,       // Lowest priority (largest data)
        }
    }
}

/// Trait for data types that can be synchronized
///
/// Implementations of this trait define how a particular data source
/// tracks changes and applies updates from remote sources.
#[async_trait]
pub trait SyncableData: Send + Sync {
    /// Get all changes since the given timestamp
    ///
    /// Returns a list of changes that have occurred since the specified time.
    /// Used for incremental (delta) sync operations.
    async fn get_changes_since(&self, timestamp: DateTime<Utc>) -> SyncResult<Vec<Change>>;

    /// Apply a list of changes from a remote source
    ///
    /// This method should handle conflicts according to the configured strategy
    /// and return the number of successfully applied changes.
    async fn apply_changes(&mut self, changes: Vec<Change>) -> SyncResult<usize>;

    /// Get a unique key identifying this syncable data source
    ///
    /// Used to identify the data source in sync operations and conflict resolution.
    fn get_sync_key(&self) -> String;

    /// Get the data type this source represents
    fn data_type(&self) -> SyncDataType;

    /// Get all data for initial sync
    ///
    /// Used when syncing a new device that needs to download all existing data.
    async fn get_all_data(&self) -> SyncResult<Vec<Change>>;

    /// Clear all data and reset to empty state
    ///
    /// Used when user logs out or wants to clear synced data.
    async fn clear_sync_data(&mut self) -> SyncResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_data_type_all() {
        let all = SyncDataType::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&SyncDataType::Bookmarks));
        assert!(all.contains(&SyncDataType::History));
        assert!(all.contains(&SyncDataType::Settings));
        assert!(all.contains(&SyncDataType::Passwords));
        assert!(all.contains(&SyncDataType::OpenTabs));
    }

    #[test]
    fn test_requires_encryption() {
        assert!(!SyncDataType::Bookmarks.requires_encryption());
        assert!(!SyncDataType::History.requires_encryption());
        assert!(SyncDataType::Settings.requires_encryption());
        assert!(SyncDataType::Passwords.requires_encryption());
        assert!(!SyncDataType::OpenTabs.requires_encryption());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(SyncDataType::Settings.priority() < SyncDataType::Passwords.priority());
        assert!(SyncDataType::Passwords.priority() < SyncDataType::Bookmarks.priority());
        assert!(SyncDataType::Bookmarks.priority() < SyncDataType::History.priority());
    }

    #[test]
    fn test_serialization() {
        let dt = SyncDataType::Bookmarks;
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, "\"bookmarks\"");

        let parsed: SyncDataType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SyncDataType::Bookmarks);
    }
}
