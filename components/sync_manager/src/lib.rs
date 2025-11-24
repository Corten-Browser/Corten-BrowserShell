//! Sync Manager Component
//!
//! Provides cross-device synchronization infrastructure for the CortenBrowser Browser Shell.
//! Supports syncing bookmarks, history, settings, passwords, and open tabs across devices.
//!
//! # Features
//! - Sync account management (login/logout)
//! - Multiple data type support (bookmarks, history, settings, passwords, open tabs)
//! - Conflict resolution strategies (last-write-wins, merge)
//! - Incremental sync with delta updates
//! - Offline queue for changes made while disconnected
//! - End-to-end encryption for synced data

mod account;
mod change;
mod conflict;
mod encryption;
mod manager;
mod offline_queue;
mod status;
mod syncable;

pub use account::{SyncAccount, SyncAccountCredentials};
pub use change::{Change, ChangeOperation, ChangeId};
pub use conflict::{ConflictResolution, ConflictStrategy};
pub use encryption::{EncryptedData, EncryptionKey, SyncEncryption};
pub use manager::SyncManager;
pub use offline_queue::{OfflineQueue, QueuedChange};
pub use status::{SyncError, SyncOperationResult, SyncResult, SyncStatus, SyncState, TypeSyncStatus};
pub use syncable::{SyncDataType, SyncableData};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_sync_data_type_display() {
        assert_eq!(format!("{}", SyncDataType::Bookmarks), "bookmarks");
        assert_eq!(format!("{}", SyncDataType::History), "history");
        assert_eq!(format!("{}", SyncDataType::Settings), "settings");
        assert_eq!(format!("{}", SyncDataType::Passwords), "passwords");
        assert_eq!(format!("{}", SyncDataType::OpenTabs), "open_tabs");
    }

    #[test]
    fn test_change_creation() {
        let change = Change::new(
            SyncDataType::Bookmarks,
            "bookmark_123".to_string(),
            ChangeOperation::Create,
            serde_json::json!({"url": "https://example.com", "title": "Example"}),
        );

        assert_eq!(change.data_type, SyncDataType::Bookmarks);
        assert_eq!(change.entity_id, "bookmark_123");
        assert!(matches!(change.operation, ChangeOperation::Create));
    }

    #[test]
    fn test_sync_status_default() {
        let status = SyncStatus::default();
        assert!(matches!(status.state, SyncState::Idle));
        assert!(status.last_sync.is_none());
        assert_eq!(status.pending_changes, 0);
    }

    #[test]
    fn test_conflict_resolution_last_write_wins() {
        let local = Change::new(
            SyncDataType::Settings,
            "setting_1".to_string(),
            ChangeOperation::Update,
            serde_json::json!({"value": "local"}),
        );

        let mut remote = Change::new(
            SyncDataType::Settings,
            "setting_1".to_string(),
            ChangeOperation::Update,
            serde_json::json!({"value": "remote"}),
        );
        // Make remote newer
        remote.timestamp = Utc::now() + chrono::Duration::seconds(10);

        let resolver = ConflictResolution::new(ConflictStrategy::LastWriteWins);
        let resolved = resolver.resolve(&local, &remote);

        // Remote should win (newer timestamp)
        assert_eq!(resolved.data, serde_json::json!({"value": "remote"}));
    }

    #[tokio::test]
    async fn test_offline_queue() {
        let queue = OfflineQueue::new();

        let change = Change::new(
            SyncDataType::Bookmarks,
            "bookmark_1".to_string(),
            ChangeOperation::Create,
            serde_json::json!({"url": "https://example.com"}),
        );

        queue.enqueue(change.clone()).await;
        assert_eq!(queue.len().await, 1);

        let pending = queue.drain().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].change.entity_id, "bookmark_1");
        assert_eq!(queue.len().await, 0);
    }

    #[tokio::test]
    async fn test_sync_manager_not_logged_in() {
        let manager = SyncManager::new();

        let result = manager.sync(vec![SyncDataType::Bookmarks]).await;
        assert!(result.is_err());

        if let Err(SyncError::NotLoggedIn) = result {
            // Expected
        } else {
            panic!("Expected NotLoggedIn error");
        }
    }

    #[test]
    fn test_encryption_key_derivation() {
        let key = EncryptionKey::derive_from_password("test_password", "test@example.com");
        assert!(!key.as_bytes().is_empty());

        // Same input should produce same key
        let key2 = EncryptionKey::derive_from_password("test_password", "test@example.com");
        assert_eq!(key.as_bytes(), key2.as_bytes());

        // Different password should produce different key
        let key3 = EncryptionKey::derive_from_password("other_password", "test@example.com");
        assert_ne!(key.as_bytes(), key3.as_bytes());
    }
}
