//! Downloads Manager Component
//!
//! This component provides download tracking and management functionality for the
//! CortenBrowser Browser Shell. It handles starting, pausing, resuming, and cancelling
//! downloads, as well as tracking download progress.

use serde::{Deserialize, Serialize};
use shared_types::{ComponentError, DownloadId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use url::Url;

/// Status of a download
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// Download is pending (not yet started)
    Pending,
    /// Download is in progress
    Downloading,
    /// Download is paused
    Paused,
    /// Download completed successfully
    Complete,
    /// Download failed with an error message
    Failed(String),
    /// Download was cancelled by the user
    Cancelled,
}

/// Information about a download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// Unique identifier for this download
    pub id: DownloadId,
    /// URL being downloaded
    pub url: String,
    /// Destination path on disk
    pub destination: String,
    /// Filename extracted from URL or destination
    pub filename: String,
    /// Total size in bytes (0 if unknown)
    pub total_bytes: u64,
    /// Number of bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Current download status
    pub status: DownloadStatus,
}

/// Internal download task state
struct DownloadTask {
    /// Download metadata
    info: Arc<RwLock<DownloadInfo>>,
    /// Handle to the download task
    task_handle: Option<JoinHandle<()>>,
    /// Channel for pause/resume/cancel signals
    control_tx: tokio::sync::mpsc::Sender<ControlSignal>,
}

/// Control signals for download tasks
#[derive(Debug, Clone, Copy)]
enum ControlSignal {
    Pause,
    Resume,
    Cancel,
}

/// Downloads manager that tracks and manages file downloads
pub struct DownloadsManager {
    /// Map of download ID to download task
    downloads: Arc<Mutex<HashMap<DownloadId, DownloadTask>>>,
}

impl DownloadsManager {
    /// Create a new DownloadsManager
    pub fn new() -> Self {
        Self {
            downloads: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start downloading a file
    ///
    /// # Arguments
    /// * `url` - URL to download from
    /// * `destination` - Optional destination path. If None, uses a default downloads directory
    ///
    /// # Returns
    /// * `Ok(DownloadId)` - ID of the started download
    /// * `Err(ComponentError)` - If the URL is invalid or download cannot be started
    pub async fn start_download(
        &self,
        url: String,
        destination: Option<String>,
    ) -> Result<DownloadId, ComponentError> {
        // Validate URL
        let parsed_url = Url::parse(&url).map_err(|e| {
            ComponentError::InvalidState(format!("Invalid URL: {}", e))
        })?;

        // Extract filename from URL
        let filename = Self::extract_filename(&parsed_url);

        // Determine destination path
        let dest_path = destination.unwrap_or_else(|| {
            format!("/downloads/{}", filename)
        });

        // Create download info
        let download_id = DownloadId::new();
        let info = Arc::new(RwLock::new(DownloadInfo {
            id: download_id,
            url: url.clone(),
            destination: dest_path.clone(),
            filename: filename.clone(),
            total_bytes: 1024 * 1024, // Mock: 1MB file
            downloaded_bytes: 0,
            status: DownloadStatus::Pending,
        }));

        // Create control channel
        let (control_tx, control_rx) = tokio::sync::mpsc::channel(10);

        // Spawn download task
        let info_clone = info.clone();
        let task_handle = tokio::spawn(async move {
            Self::download_task(info_clone, control_rx).await;
        });

        // Store download task
        let task = DownloadTask {
            info: info.clone(),
            task_handle: Some(task_handle),
            control_tx,
        };

        let mut downloads = self.downloads.lock().await;
        downloads.insert(download_id, task);

        Ok(download_id)
    }

    /// Pause an active download
    ///
    /// # Arguments
    /// * `id` - ID of the download to pause
    ///
    /// # Returns
    /// * `Ok(())` - Download successfully paused
    /// * `Err(ComponentError)` - If download doesn't exist or cannot be paused
    pub async fn pause_download(&self, id: DownloadId) -> Result<(), ComponentError> {
        let downloads = self.downloads.lock().await;
        let task = downloads
            .get(&id)
            .ok_or_else(|| ComponentError::ResourceNotFound(format!("Download {:?} not found", id)))?;

        task.control_tx
            .send(ControlSignal::Pause)
            .await
            .map_err(|e| ComponentError::InvalidState(format!("Failed to pause download: {}", e)))?;

        Ok(())
    }

    /// Resume a paused download
    ///
    /// # Arguments
    /// * `id` - ID of the download to resume
    ///
    /// # Returns
    /// * `Ok(())` - Download successfully resumed
    /// * `Err(ComponentError)` - If download doesn't exist or cannot be resumed
    pub async fn resume_download(&self, id: DownloadId) -> Result<(), ComponentError> {
        let downloads = self.downloads.lock().await;
        let task = downloads
            .get(&id)
            .ok_or_else(|| ComponentError::ResourceNotFound(format!("Download {:?} not found", id)))?;

        task.control_tx
            .send(ControlSignal::Resume)
            .await
            .map_err(|e| ComponentError::InvalidState(format!("Failed to resume download: {}", e)))?;

        Ok(())
    }

    /// Cancel a download
    ///
    /// # Arguments
    /// * `id` - ID of the download to cancel
    ///
    /// # Returns
    /// * `Ok(())` - Download successfully cancelled
    /// * `Err(ComponentError)` - If download doesn't exist or cannot be cancelled
    pub async fn cancel_download(&self, id: DownloadId) -> Result<(), ComponentError> {
        let downloads = self.downloads.lock().await;
        let task = downloads
            .get(&id)
            .ok_or_else(|| ComponentError::ResourceNotFound(format!("Download {:?} not found", id)))?;

        task.control_tx
            .send(ControlSignal::Cancel)
            .await
            .map_err(|e| ComponentError::InvalidState(format!("Failed to cancel download: {}", e)))?;

        Ok(())
    }

    /// Get download information
    ///
    /// # Arguments
    /// * `id` - ID of the download
    ///
    /// # Returns
    /// * `Some(DownloadInfo)` - Information about the download
    /// * `None` - If download doesn't exist
    pub async fn get_download_info(&self, id: DownloadId) -> Option<DownloadInfo> {
        let downloads = self.downloads.lock().await;
        let task = downloads.get(&id)?;
        let info = task.info.read().await;
        Some(info.clone())
    }

    /// Get all active downloads
    ///
    /// Active downloads are those that are not cancelled or completed.
    ///
    /// # Returns
    /// Vector of DownloadInfo for all active downloads
    pub async fn get_active_downloads(&self) -> Vec<DownloadInfo> {
        let downloads = self.downloads.lock().await;
        let mut active = Vec::new();

        for task in downloads.values() {
            let info = task.info.read().await;
            match info.status {
                DownloadStatus::Pending
                | DownloadStatus::Downloading
                | DownloadStatus::Paused => {
                    active.push(info.clone());
                }
                _ => {}
            }
        }

        active
    }

    /// Extract filename from URL
    fn extract_filename(url: &Url) -> String {
        url.path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("download")
            .to_string()
    }

    /// Mock download task that simulates downloading a file
    ///
    /// This is a simulated download for testing purposes. In a real implementation,
    /// this would use HTTP client to actually download files.
    async fn download_task(
        info: Arc<RwLock<DownloadInfo>>,
        mut control_rx: tokio::sync::mpsc::Receiver<ControlSignal>,
    ) {
        // Set status to downloading
        {
            let mut info_write = info.write().await;
            info_write.status = DownloadStatus::Downloading;
        }

        let total_bytes = {
            let info_read = info.read().await;
            info_read.total_bytes
        };

        let chunk_size = 1024 * 10; // 10KB chunks
        let delay = tokio::time::Duration::from_millis(10); // Simulate network delay
        let check_interval = tokio::time::Duration::from_millis(1); // Check control signals frequently

        let mut downloaded = 0u64;
        let mut paused = false;

        loop {
            // Check for control signals with timeout (allows frequent checking)
            match tokio::time::timeout(check_interval, control_rx.recv()).await {
                Ok(Some(ControlSignal::Pause)) => {
                    paused = true;
                    let mut info_write = info.write().await;
                    info_write.status = DownloadStatus::Paused;
                }
                Ok(Some(ControlSignal::Resume)) => {
                    paused = false;
                    let mut info_write = info.write().await;
                    info_write.status = DownloadStatus::Downloading;
                }
                Ok(Some(ControlSignal::Cancel)) => {
                    let mut info_write = info.write().await;
                    info_write.status = DownloadStatus::Cancelled;
                    return;
                }
                Ok(None) => {
                    // Channel closed
                    return;
                }
                Err(_) => {
                    // Timeout - no signal received, continue normally
                }
            }

            // If paused, don't download
            if paused {
                tokio::time::sleep(delay).await;
                continue;
            }

            // Download a chunk
            let chunk = std::cmp::min(chunk_size, total_bytes - downloaded);
            downloaded += chunk;

            // Update progress
            {
                let mut info_write = info.write().await;
                info_write.downloaded_bytes = downloaded;
            }

            // Check if complete
            if downloaded >= total_bytes {
                let mut info_write = info.write().await;
                info_write.status = DownloadStatus::Complete;
                return;
            }

            // Simulate network delay
            tokio::time::sleep(delay).await;
        }
    }
}

impl Default for DownloadsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename() {
        let url = Url::parse("https://example.com/path/to/file.zip").unwrap();
        assert_eq!(DownloadsManager::extract_filename(&url), "file.zip");

        let url = Url::parse("https://example.com/document.pdf").unwrap();
        assert_eq!(DownloadsManager::extract_filename(&url), "document.pdf");

        let url = Url::parse("https://example.com/").unwrap();
        assert_eq!(DownloadsManager::extract_filename(&url), "download");
    }

    #[test]
    fn test_download_status_serialization() {
        let status = DownloadStatus::Downloading;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: DownloadStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_download_info_serialization() {
        let info = DownloadInfo {
            id: DownloadId::new(),
            url: "https://example.com/file.zip".to_string(),
            destination: "/downloads/file.zip".to_string(),
            filename: "file.zip".to_string(),
            total_bytes: 1024,
            downloaded_bytes: 512,
            status: DownloadStatus::Downloading,
        };

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: DownloadInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(info.url, deserialized.url);
        assert_eq!(info.filename, deserialized.filename);
    }
}
