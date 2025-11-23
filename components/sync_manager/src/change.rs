//! Change tracking for sync operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::SyncDataType;

/// Unique identifier for a change
pub type ChangeId = Uuid;

/// Type of operation performed on an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOperation {
    /// Entity was created
    Create,
    /// Entity was updated
    Update,
    /// Entity was deleted
    Delete,
}

/// Represents a single change to sync data
///
/// Changes are the fundamental unit of synchronization. Each change represents
/// a single operation (create, update, delete) on a single entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Unique identifier for this change
    pub id: ChangeId,

    /// Type of data being changed
    pub data_type: SyncDataType,

    /// ID of the entity being changed (e.g., bookmark ID, setting key)
    pub entity_id: String,

    /// Type of operation
    pub operation: ChangeOperation,

    /// The data payload (JSON serialized)
    pub data: serde_json::Value,

    /// When this change occurred
    pub timestamp: DateTime<Utc>,

    /// Device ID where this change originated
    pub device_id: String,

    /// Version number for conflict detection (monotonically increasing)
    pub version: u64,

    /// Hash of the previous state (for conflict detection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_hash: Option<String>,
}

impl Change {
    /// Create a new change
    pub fn new(
        data_type: SyncDataType,
        entity_id: String,
        operation: ChangeOperation,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            data_type,
            entity_id,
            operation,
            data,
            timestamp: Utc::now(),
            device_id: get_device_id(),
            version: 1,
            previous_hash: None,
        }
    }

    /// Create a change with a specific device ID
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = device_id;
        self
    }

    /// Create a change with a specific version
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }

    /// Create a change with a previous hash
    pub fn with_previous_hash(mut self, hash: String) -> Self {
        self.previous_hash = Some(hash);
        self
    }

    /// Check if this change conflicts with another change
    pub fn conflicts_with(&self, other: &Change) -> bool {
        self.entity_id == other.entity_id
            && self.data_type == other.data_type
            && self.id != other.id
    }

    /// Check if this is a delete operation
    pub fn is_delete(&self) -> bool {
        matches!(self.operation, ChangeOperation::Delete)
    }

    /// Check if this is a create operation
    pub fn is_create(&self) -> bool {
        matches!(self.operation, ChangeOperation::Create)
    }

    /// Get a sortable key for this change (for ordering)
    pub fn sort_key(&self) -> (DateTime<Utc>, u64) {
        (self.timestamp, self.version)
    }
}

/// Get the current device ID
fn get_device_id() -> String {
    // In a real implementation, this would be a persistent device identifier
    // For now, use a placeholder that could be replaced with actual device ID
    std::env::var("CORTEN_DEVICE_ID").unwrap_or_else(|_| "unknown_device".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_creation() {
        let change = Change::new(
            SyncDataType::Bookmarks,
            "bm_123".to_string(),
            ChangeOperation::Create,
            serde_json::json!({"url": "https://example.com"}),
        );

        assert_eq!(change.entity_id, "bm_123");
        assert!(change.is_create());
        assert!(!change.is_delete());
    }

    #[test]
    fn test_change_with_device_id() {
        let change = Change::new(
            SyncDataType::Settings,
            "setting_1".to_string(),
            ChangeOperation::Update,
            serde_json::json!({}),
        )
        .with_device_id("device_abc".to_string());

        assert_eq!(change.device_id, "device_abc");
    }

    #[test]
    fn test_change_conflicts() {
        let change1 = Change::new(
            SyncDataType::Bookmarks,
            "bm_123".to_string(),
            ChangeOperation::Update,
            serde_json::json!({"title": "Updated"}),
        );

        let change2 = Change::new(
            SyncDataType::Bookmarks,
            "bm_123".to_string(),
            ChangeOperation::Update,
            serde_json::json!({"title": "Also Updated"}),
        );

        let change3 = Change::new(
            SyncDataType::Bookmarks,
            "bm_456".to_string(),
            ChangeOperation::Update,
            serde_json::json!({"title": "Different"}),
        );

        assert!(change1.conflicts_with(&change2));
        assert!(!change1.conflicts_with(&change3));
        assert!(!change1.conflicts_with(&change1)); // Same change doesn't conflict with itself
    }

    #[test]
    fn test_change_serialization() {
        let change = Change::new(
            SyncDataType::History,
            "hist_1".to_string(),
            ChangeOperation::Delete,
            serde_json::json!(null),
        );

        let json = serde_json::to_string(&change).unwrap();
        let parsed: Change = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.entity_id, change.entity_id);
        assert_eq!(parsed.data_type, change.data_type);
        assert!(parsed.is_delete());
    }
}
