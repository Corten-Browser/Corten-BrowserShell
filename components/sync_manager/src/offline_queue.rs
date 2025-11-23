//! Offline queue for changes made while disconnected

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::Change;

/// A change that has been queued for sync when online
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedChange {
    /// Unique ID for this queued item
    pub id: Uuid,

    /// The actual change to sync
    pub change: Change,

    /// When this change was queued
    pub queued_at: DateTime<Utc>,

    /// Number of sync attempts made
    pub attempts: u32,

    /// Last error message if sync failed
    pub last_error: Option<String>,

    /// Priority (lower = higher priority)
    pub priority: u8,
}

impl QueuedChange {
    /// Create a new queued change
    pub fn new(change: Change) -> Self {
        let priority = change.data_type.priority();
        Self {
            id: Uuid::new_v4(),
            change,
            queued_at: Utc::now(),
            attempts: 0,
            last_error: None,
            priority,
        }
    }

    /// Mark a failed sync attempt
    pub fn mark_failed(&mut self, error: String) {
        self.attempts += 1;
        self.last_error = Some(error);
    }

    /// Check if this change should be retried
    pub fn should_retry(&self, max_attempts: u32) -> bool {
        self.attempts < max_attempts
    }
}

/// Queue for storing changes made while offline
///
/// Changes are stored in memory and can be persisted to disk.
/// When connectivity is restored, changes are synced in order.
#[derive(Debug)]
pub struct OfflineQueue {
    queue: RwLock<VecDeque<QueuedChange>>,
    max_size: usize,
    max_attempts: u32,
}

impl OfflineQueue {
    /// Create a new offline queue with default settings
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            max_size: 10000,
            max_attempts: 5,
        }
    }

    /// Create a queue with custom settings
    pub fn with_settings(max_size: usize, max_attempts: u32) -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            max_size,
            max_attempts,
        }
    }

    /// Add a change to the queue
    pub async fn enqueue(&self, change: Change) {
        let mut queue = self.queue.write().await;

        // Check if we're at capacity
        if queue.len() >= self.max_size {
            // Remove oldest low-priority items to make room
            self.evict_if_needed(&mut queue);
        }

        let queued = QueuedChange::new(change);
        queue.push_back(queued);

        // Sort by priority
        self.sort_by_priority(&mut queue);
    }

    /// Get the number of queued changes
    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Check if the queue is empty
    pub async fn is_empty(&self) -> bool {
        self.queue.read().await.is_empty()
    }

    /// Get all queued changes without removing them
    pub async fn peek_all(&self) -> Vec<QueuedChange> {
        self.queue.read().await.iter().cloned().collect()
    }

    /// Get changes that should be retried
    pub async fn get_pending(&self) -> Vec<QueuedChange> {
        self.queue
            .read()
            .await
            .iter()
            .filter(|c| c.should_retry(self.max_attempts))
            .cloned()
            .collect()
    }

    /// Remove and return all queued changes
    pub async fn drain(&self) -> Vec<QueuedChange> {
        let mut queue = self.queue.write().await;
        queue.drain(..).collect()
    }

    /// Remove a specific change by ID
    pub async fn remove(&self, id: Uuid) -> Option<QueuedChange> {
        let mut queue = self.queue.write().await;
        if let Some(pos) = queue.iter().position(|c| c.id == id) {
            queue.remove(pos)
        } else {
            None
        }
    }

    /// Mark a change as failed
    pub async fn mark_failed(&self, id: Uuid, error: String) {
        let mut queue = self.queue.write().await;
        if let Some(change) = queue.iter_mut().find(|c| c.id == id) {
            change.mark_failed(error);
        }
    }

    /// Remove changes that have exceeded max attempts
    pub async fn prune_failed(&self) -> Vec<QueuedChange> {
        let mut queue = self.queue.write().await;
        let mut failed = Vec::new();

        queue.retain(|c| {
            if c.attempts >= self.max_attempts {
                failed.push(c.clone());
                false
            } else {
                true
            }
        });

        failed
    }

    /// Clear all queued changes
    pub async fn clear(&self) {
        self.queue.write().await.clear();
    }

    /// Evict low-priority items when at capacity
    fn evict_if_needed(&self, queue: &mut VecDeque<QueuedChange>) {
        // Remove 10% of oldest low-priority items
        let to_remove = queue.len() / 10;
        if to_remove == 0 {
            return;
        }

        // Sort by priority (descending) and age (ascending)
        let mut items: Vec<_> = queue.drain(..).collect();
        items.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.queued_at.cmp(&b.queued_at))
        });

        // Remove the lowest priority oldest items
        items.truncate(items.len().saturating_sub(to_remove));

        // Restore remaining items
        for item in items {
            queue.push_back(item);
        }
    }

    /// Sort queue by priority
    fn sort_by_priority(&self, queue: &mut VecDeque<QueuedChange>) {
        let mut items: Vec<_> = queue.drain(..).collect();
        items.sort_by_key(|c| (c.priority, c.queued_at));
        for item in items {
            queue.push_back(item);
        }
    }

    /// Serialize queue to JSON for persistence
    pub async fn to_json(&self) -> Result<String, serde_json::Error> {
        let queue = self.queue.read().await;
        let items: Vec<_> = queue.iter().collect();
        serde_json::to_string(&items)
    }

    /// Load queue from JSON
    pub async fn from_json(&self, json: &str) -> Result<(), serde_json::Error> {
        let items: Vec<QueuedChange> = serde_json::from_str(json)?;
        let mut queue = self.queue.write().await;
        queue.clear();
        for item in items {
            queue.push_back(item);
        }
        Ok(())
    }
}

impl Default for OfflineQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChangeOperation, SyncDataType};

    fn make_change(data_type: SyncDataType, entity_id: &str) -> Change {
        Change::new(
            data_type,
            entity_id.to_string(),
            ChangeOperation::Create,
            serde_json::json!({}),
        )
    }

    #[tokio::test]
    async fn test_enqueue_and_drain() {
        let queue = OfflineQueue::new();

        queue.enqueue(make_change(SyncDataType::Bookmarks, "bm_1")).await;
        queue.enqueue(make_change(SyncDataType::Settings, "s_1")).await;

        assert_eq!(queue.len().await, 2);

        let drained = queue.drain().await;
        assert_eq!(drained.len(), 2);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let queue = OfflineQueue::new();

        // Add in reverse priority order
        queue.enqueue(make_change(SyncDataType::History, "h_1")).await;    // Priority 5
        queue.enqueue(make_change(SyncDataType::Settings, "s_1")).await;  // Priority 1
        queue.enqueue(make_change(SyncDataType::Bookmarks, "bm_1")).await; // Priority 3

        let items = queue.peek_all().await;

        // Should be sorted by priority (1, 3, 5)
        assert_eq!(items[0].change.data_type, SyncDataType::Settings);
        assert_eq!(items[1].change.data_type, SyncDataType::Bookmarks);
        assert_eq!(items[2].change.data_type, SyncDataType::History);
    }

    #[tokio::test]
    async fn test_mark_failed() {
        let queue = OfflineQueue::new();

        queue.enqueue(make_change(SyncDataType::Bookmarks, "bm_1")).await;
        let items = queue.peek_all().await;
        let id = items[0].id;

        queue.mark_failed(id, "Network error".to_string()).await;

        let items = queue.peek_all().await;
        assert_eq!(items[0].attempts, 1);
        assert_eq!(items[0].last_error, Some("Network error".to_string()));
    }

    #[tokio::test]
    async fn test_prune_failed() {
        let queue = OfflineQueue::with_settings(100, 2);

        queue.enqueue(make_change(SyncDataType::Bookmarks, "bm_1")).await;
        let items = queue.peek_all().await;
        let id = items[0].id;

        // Mark as failed twice (exceeds max_attempts of 2)
        queue.mark_failed(id, "Error 1".to_string()).await;
        queue.mark_failed(id, "Error 2".to_string()).await;

        let failed = queue.prune_failed().await;
        assert_eq!(failed.len(), 1);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_remove_by_id() {
        let queue = OfflineQueue::new();

        queue.enqueue(make_change(SyncDataType::Bookmarks, "bm_1")).await;
        queue.enqueue(make_change(SyncDataType::Settings, "s_1")).await;

        let items = queue.peek_all().await;
        let id_to_remove = items[0].id;

        let removed = queue.remove(id_to_remove).await;
        assert!(removed.is_some());
        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_serialization() {
        let queue = OfflineQueue::new();

        queue.enqueue(make_change(SyncDataType::Bookmarks, "bm_1")).await;
        queue.enqueue(make_change(SyncDataType::Settings, "s_1")).await;

        let json = queue.to_json().await.unwrap();

        let queue2 = OfflineQueue::new();
        queue2.from_json(&json).await.unwrap();

        assert_eq!(queue2.len().await, 2);
    }
}
