//! Main sync manager implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::{
    Change, ConflictResolution, ConflictStrategy, EncryptionKey, OfflineQueue,
    SyncAccount, SyncAccountCredentials, SyncDataType, SyncEncryption, SyncError,
    SyncOperationResult, SyncResult, SyncState, SyncStatus, SyncableData,
};

/// Main sync manager for cross-device synchronization
///
/// The SyncManager coordinates all sync operations including:
/// - Account management (login/logout)
/// - Data synchronization across devices
/// - Conflict resolution
/// - Offline change queuing
/// - Encryption of sensitive data
pub struct SyncManager {
    /// Current logged-in account
    account: RwLock<Option<SyncAccount>>,

    /// Authentication credentials
    credentials: RwLock<Option<SyncAccountCredentials>>,

    /// Current sync status
    status: RwLock<SyncStatus>,

    /// Offline change queue
    offline_queue: Arc<OfflineQueue>,

    /// Conflict resolution strategy
    conflict_resolver: RwLock<ConflictResolution>,

    /// Encryption handler (set after login)
    encryption: RwLock<Option<SyncEncryption>>,

    /// Registered syncable data sources
    data_sources: RwLock<HashMap<SyncDataType, Arc<dyn SyncableData>>>,

    /// Last sync timestamps per data type
    last_sync_times: RwLock<HashMap<SyncDataType, DateTime<Utc>>>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new() -> Self {
        Self {
            account: RwLock::new(None),
            credentials: RwLock::new(None),
            status: RwLock::new(SyncStatus::default()),
            offline_queue: Arc::new(OfflineQueue::new()),
            conflict_resolver: RwLock::new(ConflictResolution::default()),
            encryption: RwLock::new(None),
            data_sources: RwLock::new(HashMap::new()),
            last_sync_times: RwLock::new(HashMap::new()),
        }
    }

    /// Create a sync manager with a specific conflict strategy
    pub fn with_conflict_strategy(strategy: ConflictStrategy) -> Self {
        let mut manager = Self::new();
        // Use blocking lock since we're in constructor
        *manager.conflict_resolver.get_mut() = ConflictResolution::new(strategy);
        manager
    }

    /// Log in to a sync account
    ///
    /// # Arguments
    /// * `account` - The sync account to log into
    /// * `credentials` - Authentication credentials
    /// * `password` - Password for encryption key derivation
    pub async fn login(
        &self,
        account: SyncAccount,
        credentials: SyncAccountCredentials,
        password: &str,
    ) -> SyncResult<()> {
        // Derive encryption key from password and email
        let key = EncryptionKey::derive_from_password(password, &account.email);
        let encryption = SyncEncryption::new(key);

        // Store account and credentials
        *self.account.write().await = Some(account);
        *self.credentials.write().await = Some(credentials);
        *self.encryption.write().await = Some(encryption);

        // Update status
        let mut status = self.status.write().await;
        status.is_enabled = true;

        Ok(())
    }

    /// Log out of the current sync account
    pub async fn logout(&self) -> SyncResult<()> {
        // Clear account and credentials
        *self.account.write().await = None;
        *self.credentials.write().await = None;
        *self.encryption.write().await = None;

        // Clear offline queue
        self.offline_queue.clear().await;

        // Update status
        let mut status = self.status.write().await;
        status.is_enabled = false;
        status.state = SyncState::Idle;
        status.pending_changes = 0;

        Ok(())
    }

    /// Check if user is logged in
    pub async fn is_logged_in(&self) -> bool {
        self.account.read().await.is_some()
    }

    /// Get the current account (if logged in)
    pub async fn get_account(&self) -> Option<SyncAccount> {
        self.account.read().await.clone()
    }

    /// Register a data source for synchronization
    pub async fn register_data_source(&self, source: Arc<dyn SyncableData>) {
        let data_type = source.data_type();
        self.data_sources.write().await.insert(data_type, source);
    }

    /// Sync specific data types
    ///
    /// # Arguments
    /// * `types` - Data types to sync. If empty, syncs all registered types.
    pub async fn sync(&self, types: Vec<SyncDataType>) -> SyncResult<SyncOperationResult> {
        // Verify logged in
        if !self.is_logged_in().await {
            return Err(SyncError::NotLoggedIn);
        }

        // Check if already syncing
        {
            let status = self.status.read().await;
            if status.is_syncing() {
                return Err(SyncError::SyncInProgress);
            }
        }

        let start_time = std::time::Instant::now();

        // Update status to syncing
        {
            let mut status = self.status.write().await;
            status.set_state(SyncState::Checking);
        }

        // Determine which types to sync
        let types_to_sync: Vec<SyncDataType> = {
            let account = self.account.read().await;
            let account = match account.as_ref() {
                Some(acc) => acc,
                None => return Err(SyncError::NotLoggedIn),
            };

            if types.is_empty() {
                account.device_settings.enabled_types.clone()
            } else {
                types
                    .into_iter()
                    .filter(|t| account.is_type_enabled(*t))
                    .collect()
            }
        };

        let mut total_uploaded = 0;
        let mut total_downloaded = 0;
        let mut total_conflicts = 0;

        // Process offline queue first
        let queued_changes = self.offline_queue.get_pending().await;
        if !queued_changes.is_empty() {
            // In a real implementation, this would upload queued changes to server
            total_uploaded += queued_changes.len();

            // Clear successfully synced changes
            for change in &queued_changes {
                self.offline_queue.remove(change.id).await;
            }
        }

        // Sync each data type
        for data_type in &types_to_sync {
            match self.sync_data_type(*data_type).await {
                Ok((uploaded, downloaded, conflicts)) => {
                    total_uploaded += uploaded;
                    total_downloaded += downloaded;
                    total_conflicts += conflicts;
                }
                Err(e) => {
                    // Log error but continue with other types
                    let mut status = self.status.write().await;
                    status.set_state(SyncState::Error);
                    status.error_message = Some(format!("Error syncing {}: {}", data_type, e));
                }
            }
        }

        // Update status to complete
        {
            let mut status = self.status.write().await;
            status.complete();
            status.conflicts_detected = total_conflicts;
        }

        let duration = start_time.elapsed();

        Ok(SyncOperationResult::success(
            total_uploaded,
            total_downloaded,
            total_conflicts,
            duration.as_millis() as u64,
        ))
    }

    /// Sync a specific data type
    async fn sync_data_type(&self, data_type: SyncDataType) -> SyncResult<(usize, usize, usize)> {
        let data_sources = self.data_sources.read().await;
        let _source = data_sources
            .get(&data_type)
            .ok_or(SyncError::TypeNotEnabled(data_type))?;

        // Get last sync time for this type
        let _last_sync = self
            .last_sync_times
            .read()
            .await
            .get(&data_type)
            .copied()
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now));

        // In a real implementation:
        // 1. Get local changes since last sync
        // 2. Upload local changes to server
        // 3. Download remote changes
        // 4. Detect and resolve conflicts
        // 5. Apply resolved changes locally

        // For now, simulate a sync operation
        let uploaded = 0;
        let downloaded = 0;
        let conflicts = 0;

        // Update last sync time
        self.last_sync_times
            .write()
            .await
            .insert(data_type, Utc::now());

        Ok((uploaded, downloaded, conflicts))
    }

    /// Queue a change for sync when offline
    pub async fn queue_change(&self, change: Change) {
        self.offline_queue.enqueue(change).await;

        // Update pending count in status
        let mut status = self.status.write().await;
        status.pending_changes = self.offline_queue.len().await;
    }

    /// Get the current sync status
    pub async fn get_sync_status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    /// Set the conflict resolution strategy
    pub async fn set_conflict_strategy(&self, strategy: ConflictStrategy) {
        *self.conflict_resolver.write().await = ConflictResolution::new(strategy);
    }

    /// Get the offline queue
    pub fn offline_queue(&self) -> &Arc<OfflineQueue> {
        &self.offline_queue
    }

    /// Encrypt data for sync
    pub async fn encrypt(&self, data: &[u8]) -> SyncResult<crate::EncryptedData> {
        let encryption = self.encryption.read().await;
        let enc = encryption
            .as_ref()
            .ok_or(SyncError::NotLoggedIn)?;
        enc.encrypt(data)
    }

    /// Decrypt synced data
    pub async fn decrypt(&self, encrypted: &crate::EncryptedData) -> SyncResult<Vec<u8>> {
        let encryption = self.encryption.read().await;
        let enc = encryption
            .as_ref()
            .ok_or(SyncError::NotLoggedIn)?;
        enc.decrypt(encrypted)
    }

    /// Pause sync operations
    pub async fn pause(&self) {
        let mut status = self.status.write().await;
        status.set_state(SyncState::Paused);
    }

    /// Resume sync operations
    pub async fn resume(&self) {
        let mut status = self.status.write().await;
        if matches!(status.state, SyncState::Paused) {
            status.set_state(SyncState::Idle);
        }
    }

    /// Enable or disable a specific data type for sync
    pub async fn set_type_enabled(&self, data_type: SyncDataType, enabled: bool) -> SyncResult<()> {
        let mut account = self.account.write().await;
        let account = account.as_mut().ok_or(SyncError::NotLoggedIn)?;

        if enabled {
            account.enable_type(data_type);
        } else {
            account.disable_type(data_type);
        }

        Ok(())
    }

    /// Get sync status for a specific data type
    pub async fn get_type_status(&self, data_type: SyncDataType) -> SyncResult<crate::TypeSyncStatus> {
        let account = self.account.read().await;
        let account = account.as_ref().ok_or(SyncError::NotLoggedIn)?;

        let last_sync = self.last_sync_times.read().await.get(&data_type).copied();

        Ok(crate::TypeSyncStatus {
            data_type,
            last_sync,
            pending_changes: self
                .offline_queue
                .peek_all()
                .await
                .iter()
                .filter(|c| c.change.data_type == data_type)
                .count(),
            is_enabled: account.is_type_enabled(data_type),
            items_count: 0, // Would come from data source
        })
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SyncManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncManager")
            .field("offline_queue_size", &"...")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_account() -> SyncAccount {
        SyncAccount::new(
            "acc_123".to_string(),
            "test@example.com".to_string(),
            "https://sync.example.com".to_string(),
        )
    }

    fn make_credentials() -> SyncAccountCredentials {
        SyncAccountCredentials::new("test@example.com".to_string(), "token_123".to_string())
    }

    #[tokio::test]
    async fn test_login_logout() {
        let manager = SyncManager::new();

        assert!(!manager.is_logged_in().await);

        manager
            .login(make_account(), make_credentials(), "password")
            .await
            .unwrap();

        assert!(manager.is_logged_in().await);
        let account = manager.get_account().await.unwrap();
        assert_eq!(account.email, "test@example.com");

        manager.logout().await.unwrap();
        assert!(!manager.is_logged_in().await);
    }

    #[tokio::test]
    async fn test_sync_not_logged_in() {
        let manager = SyncManager::new();

        let result = manager.sync(vec![SyncDataType::Bookmarks]).await;
        assert!(matches!(result, Err(SyncError::NotLoggedIn)));
    }

    #[tokio::test]
    async fn test_queue_change() {
        let manager = SyncManager::new();

        let change = Change::new(
            SyncDataType::Bookmarks,
            "bm_1".to_string(),
            crate::ChangeOperation::Create,
            serde_json::json!({"url": "https://example.com"}),
        );

        manager.queue_change(change).await;

        let status = manager.get_sync_status().await;
        assert_eq!(status.pending_changes, 1);
    }

    #[tokio::test]
    async fn test_encryption_after_login() {
        let manager = SyncManager::new();

        manager
            .login(make_account(), make_credentials(), "password123")
            .await
            .unwrap();

        let plaintext = b"secret data";
        let encrypted = manager.encrypt(plaintext).await.unwrap();
        let decrypted = manager.decrypt(&encrypted).await.unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let manager = SyncManager::new();

        manager.pause().await;
        let status = manager.get_sync_status().await;
        assert!(matches!(status.state, SyncState::Paused));

        manager.resume().await;
        let status = manager.get_sync_status().await;
        assert!(matches!(status.state, SyncState::Idle));
    }

    #[tokio::test]
    async fn test_set_conflict_strategy() {
        let manager = SyncManager::new();

        manager
            .set_conflict_strategy(ConflictStrategy::LocalWins)
            .await;

        // Strategy is stored internally - would be used during sync
    }

    #[tokio::test]
    async fn test_type_enable_disable() {
        let manager = SyncManager::new();

        manager
            .login(make_account(), make_credentials(), "password")
            .await
            .unwrap();

        // Disable passwords sync
        manager
            .set_type_enabled(SyncDataType::Passwords, false)
            .await
            .unwrap();

        let account = manager.get_account().await.unwrap();
        assert!(!account.is_type_enabled(SyncDataType::Passwords));
        assert!(account.is_type_enabled(SyncDataType::Bookmarks));
    }
}
