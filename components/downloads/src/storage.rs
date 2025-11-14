use crate::types::{Download, DownloadId};
use anyhow::Result;

/// SQLite-based storage for download metadata
pub struct DownloadStorage {
    _db_path: String,
}

impl DownloadStorage {
    /// Create new storage with database at given path
    pub fn new(_db_path: &str) -> Result<Self> {
        unimplemented!("DownloadStorage::new not implemented")
    }

    /// Insert a new download
    pub fn insert(&self, _download: &Download) -> Result<()> {
        unimplemented!("insert not implemented")
    }

    /// Get download by ID
    pub fn get(&self, _id: &DownloadId) -> Result<Option<Download>> {
        unimplemented!("get not implemented")
    }

    /// Update existing download
    pub fn update(&self, _download: &Download) -> Result<()> {
        unimplemented!("update not implemented")
    }

    /// Delete download by ID
    pub fn delete(&self, _id: &DownloadId) -> Result<()> {
        unimplemented!("delete not implemented")
    }

    /// List all downloads
    pub fn list_all(&self) -> Result<Vec<Download>> {
        unimplemented!("list_all not implemented")
    }

    /// List downloads by status prefix (e.g., "Pending", "Completed")
    pub fn list_by_status_prefix(&self, _prefix: &str) -> Result<Vec<Download>> {
        unimplemented!("list_by_status_prefix not implemented")
    }

    /// Clear all completed downloads
    pub fn clear_completed(&self) -> Result<()> {
        unimplemented!("clear_completed not implemented")
    }
}
