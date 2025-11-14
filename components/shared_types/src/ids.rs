//! ID types for browser shell components
//!
//! This module provides strongly-typed newtype wrappers around UUIDs and process IDs
//! to prevent mixing different ID types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for browser windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(Uuid);

impl WindowId {
    /// Create a new random WindowId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for WindowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(Uuid);

impl TabId {
    /// Create a new random TabId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

/// Operating system process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(u32);

impl ProcessId {
    /// Create a ProcessId from a u32 value
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the inner u32 value
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Identifier for render surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderSurfaceId(Uuid);

impl RenderSurfaceId {
    /// Create a new random RenderSurfaceId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for RenderSurfaceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Identifier for downloads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DownloadId(Uuid);

impl DownloadId {
    /// Create a new random DownloadId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for DownloadId {
    fn default() -> Self {
        Self::new()
    }
}

/// Identifier for bookmarks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookmarkId(Uuid);

impl BookmarkId {
    /// Create a new random BookmarkId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for BookmarkId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_id_uniqueness() {
        let id1 = WindowId::new();
        let id2 = WindowId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_tab_id_uniqueness() {
        let id1 = TabId::new();
        let id2 = TabId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_process_id_equality() {
        let id1 = ProcessId::new(42);
        let id2 = ProcessId::new(42);
        assert_eq!(id1, id2);
    }
}
