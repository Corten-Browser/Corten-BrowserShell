//! Sync status and error types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::SyncDataType;

/// Result type for sync operations
pub type SyncResult<T> = Result<T, SyncError>;

/// Errors that can occur during sync operations
#[derive(Error, Debug)]
pub enum SyncError {
    /// User is not logged in
    #[error("Not logged in to sync account")]
    NotLoggedIn,

    /// Network error during sync
    #[error("Network error: {0}")]
    Network(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// Server returned an error
    #[error("Server error: {0}")]
    ServerError(String),

    /// Conflict that could not be resolved
    #[error("Unresolvable conflict for entity {entity_id}: {reason}")]
    ConflictError {
        entity_id: String,
        reason: String,
    },

    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Data serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid data received
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Rate limited by server
    #[error("Rate limited, retry after {retry_after_seconds} seconds")]
    RateLimited {
        retry_after_seconds: u64,
    },

    /// Sync is already in progress
    #[error("Sync already in progress")]
    SyncInProgress,

    /// Data type not enabled for sync
    #[error("Data type {0} is not enabled for sync")]
    TypeNotEnabled(SyncDataType),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for SyncError {
    fn from(err: serde_json::Error) -> Self {
        SyncError::SerializationError(err.to_string())
    }
}

/// Current state of the sync system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    /// Not syncing, ready for operations
    Idle,
    /// Checking for changes
    Checking,
    /// Uploading local changes
    Uploading,
    /// Downloading remote changes
    Downloading,
    /// Resolving conflicts
    ResolvingConflicts,
    /// Sync paused (user-initiated or due to network)
    Paused,
    /// Error occurred, sync stopped
    Error,
}

/// Status information about sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Current sync state
    pub state: SyncState,

    /// When the last successful sync completed
    pub last_sync: Option<DateTime<Utc>>,

    /// Number of changes waiting to be synced
    pub pending_changes: usize,

    /// Number of conflicts detected in last sync
    pub conflicts_detected: usize,

    /// Per-data-type sync status
    pub type_status: Vec<TypeSyncStatus>,

    /// Error message if state is Error
    pub error_message: Option<String>,

    /// Whether sync is enabled
    pub is_enabled: bool,

    /// Progress percentage (0-100) if syncing
    pub progress: Option<u8>,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            state: SyncState::Idle,
            last_sync: None,
            pending_changes: 0,
            conflicts_detected: 0,
            type_status: Vec::new(),
            error_message: None,
            is_enabled: true,
            progress: None,
        }
    }
}

impl SyncStatus {
    /// Create a new status with idle state
    pub fn idle() -> Self {
        Self::default()
    }

    /// Create an error status
    pub fn error(message: String) -> Self {
        Self {
            state: SyncState::Error,
            error_message: Some(message),
            ..Self::default()
        }
    }

    /// Check if sync is currently active
    pub fn is_syncing(&self) -> bool {
        matches!(
            self.state,
            SyncState::Checking
                | SyncState::Uploading
                | SyncState::Downloading
                | SyncState::ResolvingConflicts
        )
    }

    /// Check if there are pending changes
    pub fn has_pending_changes(&self) -> bool {
        self.pending_changes > 0
    }

    /// Update the state
    pub fn set_state(&mut self, state: SyncState) {
        self.state = state;
        if state != SyncState::Error {
            self.error_message = None;
        }
    }

    /// Mark sync as complete
    pub fn complete(&mut self) {
        self.state = SyncState::Idle;
        self.last_sync = Some(Utc::now());
        self.progress = None;
    }
}

/// Sync status for a specific data type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSyncStatus {
    /// The data type
    pub data_type: SyncDataType,

    /// When this type was last synced
    pub last_sync: Option<DateTime<Utc>>,

    /// Number of pending changes for this type
    pub pending_changes: usize,

    /// Whether sync is enabled for this type
    pub is_enabled: bool,

    /// Number of items synced
    pub items_count: usize,
}

impl TypeSyncStatus {
    /// Create new type status
    pub fn new(data_type: SyncDataType) -> Self {
        Self {
            data_type,
            last_sync: None,
            pending_changes: 0,
            is_enabled: true,
            items_count: 0,
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperationResult {
    /// Whether the sync was successful
    pub success: bool,

    /// Number of changes uploaded
    pub changes_uploaded: usize,

    /// Number of changes downloaded
    pub changes_downloaded: usize,

    /// Number of conflicts resolved
    pub conflicts_resolved: usize,

    /// Duration of the sync operation in milliseconds
    pub duration_ms: u64,

    /// Error message if not successful
    pub error: Option<String>,
}

impl SyncOperationResult {
    /// Create a successful result
    pub fn success(uploaded: usize, downloaded: usize, conflicts: usize, duration_ms: u64) -> Self {
        Self {
            success: true,
            changes_uploaded: uploaded,
            changes_downloaded: downloaded,
            conflicts_resolved: conflicts,
            duration_ms,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            changes_uploaded: 0,
            changes_downloaded: 0,
            conflicts_resolved: 0,
            duration_ms: 0,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_default() {
        let status = SyncStatus::default();
        assert!(matches!(status.state, SyncState::Idle));
        assert!(!status.is_syncing());
        assert!(!status.has_pending_changes());
    }

    #[test]
    fn test_sync_status_syncing() {
        let mut status = SyncStatus::default();
        status.set_state(SyncState::Uploading);
        assert!(status.is_syncing());

        status.set_state(SyncState::Paused);
        assert!(!status.is_syncing());
    }

    #[test]
    fn test_sync_status_complete() {
        let mut status = SyncStatus::default();
        status.set_state(SyncState::Uploading);
        status.progress = Some(50);
        status.complete();

        assert!(matches!(status.state, SyncState::Idle));
        assert!(status.last_sync.is_some());
        assert!(status.progress.is_none());
    }

    #[test]
    fn test_sync_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let sync_err: SyncError = json_err.into();
        assert!(matches!(sync_err, SyncError::SerializationError(_)));
    }

    #[test]
    fn test_operation_result() {
        let success = SyncOperationResult::success(10, 5, 2, 1500);
        assert!(success.success);
        assert_eq!(success.changes_uploaded, 10);
        assert_eq!(success.changes_downloaded, 5);

        let failure = SyncOperationResult::failure("Network error".to_string());
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }
}
