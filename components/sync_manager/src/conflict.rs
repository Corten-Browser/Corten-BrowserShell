//! Conflict resolution strategies for sync

use serde::{Deserialize, Serialize};

use crate::Change;

/// Strategy for resolving sync conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// Most recent change wins based on timestamp
    LastWriteWins,

    /// Local changes always win
    LocalWins,

    /// Remote changes always win
    RemoteWins,

    /// Attempt to merge changes (for compatible data types)
    Merge,

    /// Keep both versions (creates duplicate)
    KeepBoth,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        ConflictStrategy::LastWriteWins
    }
}

/// Handles conflict resolution between local and remote changes
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    /// Default strategy to use
    strategy: ConflictStrategy,
}

impl ConflictResolution {
    /// Create a new conflict resolver with the given strategy
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }

    /// Resolve a conflict between local and remote changes
    pub fn resolve(&self, local: &Change, remote: &Change) -> Change {
        match self.strategy {
            ConflictStrategy::LastWriteWins => {
                if local.timestamp > remote.timestamp {
                    local.clone()
                } else if remote.timestamp > local.timestamp {
                    remote.clone()
                } else {
                    // Same timestamp, fall back to version comparison
                    if local.version >= remote.version {
                        local.clone()
                    } else {
                        remote.clone()
                    }
                }
            }
            ConflictStrategy::LocalWins => local.clone(),
            ConflictStrategy::RemoteWins => remote.clone(),
            ConflictStrategy::Merge => self.merge_changes(local, remote),
            ConflictStrategy::KeepBoth => {
                // In a real implementation, this would create a duplicate
                // For now, prefer the newer change
                if local.timestamp >= remote.timestamp {
                    local.clone()
                } else {
                    remote.clone()
                }
            }
        }
    }

    /// Attempt to merge two changes
    fn merge_changes(&self, local: &Change, remote: &Change) -> Change {
        // Attempt to merge JSON objects
        if let (Some(local_obj), Some(remote_obj)) =
            (local.data.as_object(), remote.data.as_object())
        {
            let mut merged = local_obj.clone();

            // For each key in remote, if it doesn't exist in local or
            // remote's timestamp is newer, use remote's value
            for (key, value) in remote_obj {
                if !merged.contains_key(key) || remote.timestamp > local.timestamp {
                    merged.insert(key.clone(), value.clone());
                }
            }

            let mut result = local.clone();
            result.data = serde_json::Value::Object(merged);
            result.timestamp = std::cmp::max(local.timestamp, remote.timestamp);
            result.version = std::cmp::max(local.version, remote.version) + 1;
            result
        } else {
            // Cannot merge non-objects, fall back to last-write-wins
            if local.timestamp >= remote.timestamp {
                local.clone()
            } else {
                remote.clone()
            }
        }
    }

    /// Get the current strategy
    pub fn strategy(&self) -> ConflictStrategy {
        self.strategy
    }
}

impl Default for ConflictResolution {
    fn default() -> Self {
        Self::new(ConflictStrategy::default())
    }
}

/// Represents a detected conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// The local change
    pub local: Change,

    /// The remote change
    pub remote: Change,

    /// The resolved change (if resolution was attempted)
    pub resolved: Option<Change>,

    /// Strategy used for resolution
    pub resolution_strategy: Option<ConflictStrategy>,
}

impl Conflict {
    /// Create a new conflict
    pub fn new(local: Change, remote: Change) -> Self {
        Self {
            local,
            remote,
            resolved: None,
            resolution_strategy: None,
        }
    }

    /// Resolve this conflict with the given resolver
    pub fn resolve(&mut self, resolver: &ConflictResolution) {
        self.resolved = Some(resolver.resolve(&self.local, &self.remote));
        self.resolution_strategy = Some(resolver.strategy());
    }

    /// Check if this conflict has been resolved
    pub fn is_resolved(&self) -> bool {
        self.resolved.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChangeOperation, SyncDataType};
    use chrono::Utc;

    fn make_change(entity_id: &str, data: serde_json::Value) -> Change {
        Change::new(
            SyncDataType::Settings,
            entity_id.to_string(),
            ChangeOperation::Update,
            data,
        )
    }

    #[test]
    fn test_last_write_wins_local() {
        let mut local = make_change("setting_1", serde_json::json!({"value": "local"}));
        local.timestamp = Utc::now() + chrono::Duration::seconds(10);

        let remote = make_change("setting_1", serde_json::json!({"value": "remote"}));

        let resolver = ConflictResolution::new(ConflictStrategy::LastWriteWins);
        let resolved = resolver.resolve(&local, &remote);

        assert_eq!(resolved.data, serde_json::json!({"value": "local"}));
    }

    #[test]
    fn test_last_write_wins_remote() {
        let local = make_change("setting_1", serde_json::json!({"value": "local"}));

        let mut remote = make_change("setting_1", serde_json::json!({"value": "remote"}));
        remote.timestamp = Utc::now() + chrono::Duration::seconds(10);

        let resolver = ConflictResolution::new(ConflictStrategy::LastWriteWins);
        let resolved = resolver.resolve(&local, &remote);

        assert_eq!(resolved.data, serde_json::json!({"value": "remote"}));
    }

    #[test]
    fn test_local_wins() {
        let local = make_change("setting_1", serde_json::json!({"value": "local"}));
        let mut remote = make_change("setting_1", serde_json::json!({"value": "remote"}));
        remote.timestamp = Utc::now() + chrono::Duration::seconds(100);

        let resolver = ConflictResolution::new(ConflictStrategy::LocalWins);
        let resolved = resolver.resolve(&local, &remote);

        assert_eq!(resolved.data, serde_json::json!({"value": "local"}));
    }

    #[test]
    fn test_remote_wins() {
        let mut local = make_change("setting_1", serde_json::json!({"value": "local"}));
        local.timestamp = Utc::now() + chrono::Duration::seconds(100);
        let remote = make_change("setting_1", serde_json::json!({"value": "remote"}));

        let resolver = ConflictResolution::new(ConflictStrategy::RemoteWins);
        let resolved = resolver.resolve(&local, &remote);

        assert_eq!(resolved.data, serde_json::json!({"value": "remote"}));
    }

    #[test]
    fn test_merge_strategy() {
        let local = make_change("setting_1", serde_json::json!({"a": 1, "b": 2}));
        let mut remote = make_change("setting_1", serde_json::json!({"b": 3, "c": 4}));
        remote.timestamp = Utc::now() + chrono::Duration::seconds(5);

        let resolver = ConflictResolution::new(ConflictStrategy::Merge);
        let resolved = resolver.resolve(&local, &remote);

        // Should contain all keys, with remote values for conflicts (newer timestamp)
        let obj = resolved.data.as_object().unwrap();
        assert_eq!(obj.get("a"), Some(&serde_json::json!(1)));
        assert_eq!(obj.get("b"), Some(&serde_json::json!(3))); // Remote wins (newer)
        assert_eq!(obj.get("c"), Some(&serde_json::json!(4)));
    }

    #[test]
    fn test_conflict_resolution_flow() {
        let local = make_change("item_1", serde_json::json!({"title": "Local"}));
        let remote = make_change("item_1", serde_json::json!({"title": "Remote"}));

        let mut conflict = Conflict::new(local, remote);
        assert!(!conflict.is_resolved());

        let resolver = ConflictResolution::new(ConflictStrategy::LocalWins);
        conflict.resolve(&resolver);

        assert!(conflict.is_resolved());
        assert_eq!(
            conflict.resolved.as_ref().unwrap().data,
            serde_json::json!({"title": "Local"})
        );
    }
}
