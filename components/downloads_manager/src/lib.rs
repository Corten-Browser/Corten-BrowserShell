//! Downloads Manager Component
//!
//! This component provides download tracking and management functionality for the
//! CortenBrowser Browser Shell. It handles starting, pausing, resuming, and cancelling
//! downloads, as well as tracking download progress.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use shared_types::{ComponentError, DownloadId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
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

impl DownloadInfo {
    /// Calculate progress as a fraction (0.0 to 1.0)
    pub fn progress_fraction(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f32) / (self.total_bytes as f32)
        }
    }

    /// Calculate progress percentage (0 to 100)
    pub fn progress_percentage(&self) -> u8 {
        (self.progress_fraction() * 100.0) as u8
    }
}

/// Extended download information with real-time metrics
#[derive(Debug, Clone)]
pub struct DownloadMetrics {
    /// Basic download information
    pub info: DownloadInfo,
    /// Download speed in bytes per second (0 if not downloading)
    pub bytes_per_second: u64,
    /// Estimated time remaining in seconds (0 if unknown or complete)
    pub eta_seconds: u64,
}

/// Internal download task state
struct DownloadTask {
    /// Download metadata
    info: Arc<RwLock<DownloadInfo>>,
    /// Handle to the download task
    ///
    /// Note: This field must be kept to prevent the download task from being cancelled.
    /// When a JoinHandle is dropped, it cancels the associated task. By holding this
    /// handle, we ensure the download continues running even though we don't directly
    /// call methods on it.
    #[allow(dead_code)]
    task_handle: Option<JoinHandle<()>>,
    /// Channel for pause/resume/cancel signals
    control_tx: tokio::sync::mpsc::Sender<ControlSignal>,
    /// Time when download started
    start_time: Instant,
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
        let parsed_url = Url::parse(&url)
            .map_err(|e| ComponentError::InvalidState(format!("Invalid URL: {}", e)))?;

        // Extract filename from URL
        let filename = Self::extract_filename(&parsed_url);

        // Determine destination path
        let dest_path = destination.unwrap_or_else(|| {
            let downloads_dir = Self::get_downloads_directory();
            downloads_dir.join(&filename).to_string_lossy().to_string()
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
            start_time: Instant::now(),
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
        let task = downloads.get(&id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!("Download {:?} not found", id))
        })?;

        task.control_tx
            .send(ControlSignal::Pause)
            .await
            .map_err(|e| {
                ComponentError::InvalidState(format!("Failed to pause download: {}", e))
            })?;

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
        let task = downloads.get(&id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!("Download {:?} not found", id))
        })?;

        task.control_tx
            .send(ControlSignal::Resume)
            .await
            .map_err(|e| {
                ComponentError::InvalidState(format!("Failed to resume download: {}", e))
            })?;

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
        let task = downloads.get(&id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!("Download {:?} not found", id))
        })?;

        task.control_tx
            .send(ControlSignal::Cancel)
            .await
            .map_err(|e| {
                ComponentError::InvalidState(format!("Failed to cancel download: {}", e))
            })?;

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
                DownloadStatus::Pending | DownloadStatus::Downloading | DownloadStatus::Paused => {
                    active.push(info.clone());
                }
                _ => {}
            }
        }

        active
    }

    /// Get all downloads (including completed, failed, and cancelled)
    ///
    /// # Returns
    /// Vector of DownloadInfo for all downloads
    pub async fn get_all_downloads(&self) -> Vec<DownloadInfo> {
        let downloads = self.downloads.lock().await;
        let mut all = Vec::new();

        for task in downloads.values() {
            let info = task.info.read().await;
            all.push(info.clone());
        }

        all
    }

    /// Get download metrics with real-time speed and ETA calculations
    ///
    /// # Arguments
    /// * `id` - ID of the download
    ///
    /// # Returns
    /// * `Some(DownloadMetrics)` - Metrics for the download
    /// * `None` - If download doesn't exist
    pub async fn get_download_metrics(&self, id: DownloadId) -> Option<DownloadMetrics> {
        let downloads = self.downloads.lock().await;
        let task = downloads.get(&id)?;
        let info = task.info.read().await;

        // Calculate elapsed time
        let elapsed = task.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        // Calculate download speed (bytes per second)
        let bytes_per_second = if elapsed_secs > 0.0 && info.status == DownloadStatus::Downloading {
            (info.downloaded_bytes as f64 / elapsed_secs) as u64
        } else {
            0
        };

        // Calculate ETA (estimated time remaining in seconds)
        let eta_seconds = if bytes_per_second > 0 && info.total_bytes > info.downloaded_bytes {
            let remaining_bytes = info.total_bytes - info.downloaded_bytes;
            (remaining_bytes as f64 / bytes_per_second as f64) as u64
        } else {
            0
        };

        Some(DownloadMetrics {
            info: info.clone(),
            bytes_per_second,
            eta_seconds,
        })
    }

    /// Get metrics for all downloads
    ///
    /// # Returns
    /// Vector of DownloadMetrics for all downloads
    pub async fn get_all_download_metrics(&self) -> Vec<DownloadMetrics> {
        let downloads = self.downloads.lock().await;
        let mut metrics = Vec::new();

        for (_id, task) in downloads.iter() {
            let info = task.info.read().await;

            // Calculate elapsed time
            let elapsed = task.start_time.elapsed();
            let elapsed_secs = elapsed.as_secs_f64();

            // Calculate download speed (bytes per second)
            let bytes_per_second =
                if elapsed_secs > 0.0 && info.status == DownloadStatus::Downloading {
                    (info.downloaded_bytes as f64 / elapsed_secs) as u64
                } else {
                    0
                };

            // Calculate ETA (estimated time remaining in seconds)
            let eta_seconds = if bytes_per_second > 0 && info.total_bytes > info.downloaded_bytes {
                let remaining_bytes = info.total_bytes - info.downloaded_bytes;
                (remaining_bytes as f64 / bytes_per_second as f64) as u64
            } else {
                0
            };

            metrics.push(DownloadMetrics {
                info: info.clone(),
                bytes_per_second,
                eta_seconds,
            });
        }

        metrics
    }

    /// Extract filename from URL
    fn extract_filename(url: &Url) -> String {
        url.path_segments()
            .and_then(|mut segments| segments.next_back())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("download")
            .to_string()
    }

    /// Get the default downloads directory
    ///
    /// Returns the user's downloads directory, or a fallback if not available
    fn get_downloads_directory() -> PathBuf {
        dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
            .unwrap_or_else(|| PathBuf::from("/tmp/downloads"))
    }

    /// Ensure the parent directory of a path exists
    ///
    /// Creates all parent directories if they don't exist
    async fn ensure_parent_dir_exists(path: &Path) -> Result<(), ComponentError> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ComponentError::InvalidState(format!("Failed to create directory: {}", e))
            })?;
        }
        Ok(())
    }

    /// Download task that fetches a file from a URL and saves it to disk
    ///
    /// This implementation uses reqwest to perform actual HTTP downloads.
    /// It supports pause, resume, and cancel operations via control signals.
    ///
    /// Set DOWNLOADS_MOCK_MODE=1 environment variable to use mock downloads for testing.
    async fn download_task(
        info: Arc<RwLock<DownloadInfo>>,
        mut control_rx: tokio::sync::mpsc::Receiver<ControlSignal>,
    ) {
        // Check if mock mode is enabled (for testing)
        let mock_mode = std::env::var("DOWNLOADS_MOCK_MODE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        if mock_mode {
            Self::mock_download_task(info, control_rx).await;
            return;
        }
        // Get URL and destination from info
        let (url, destination) = {
            let info_read = info.read().await;
            (info_read.url.clone(), info_read.destination.clone())
        };

        // Set status to downloading
        {
            let mut info_write = info.write().await;
            info_write.status = DownloadStatus::Downloading;
        }

        // Ensure parent directory exists
        let dest_path = PathBuf::from(&destination);
        if let Err(e) = Self::ensure_parent_dir_exists(&dest_path).await {
            let mut info_write = info.write().await;
            info_write.status = DownloadStatus::Failed(e.to_string());
            return;
        }

        // Create HTTP client
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
        {
            Ok(client) => client,
            Err(e) => {
                let mut info_write = info.write().await;
                info_write.status =
                    DownloadStatus::Failed(format!("Failed to create HTTP client: {}", e));
                return;
            }
        };

        // Make HTTP request
        let response = match client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let mut info_write = info.write().await;
                info_write.status = DownloadStatus::Failed(format!("HTTP request failed: {}", e));
                return;
            }
        };

        // Check if request was successful
        if !response.status().is_success() {
            let mut info_write = info.write().await;
            info_write.status =
                DownloadStatus::Failed(format!("HTTP error: {}", response.status()));
            return;
        }

        // Get total size from Content-Length header
        let total_bytes = response.content_length().unwrap_or(0);
        {
            let mut info_write = info.write().await;
            info_write.total_bytes = total_bytes;
        }

        // Create file for writing
        let mut file = match tokio::fs::File::create(&dest_path).await {
            Ok(f) => f,
            Err(e) => {
                let mut info_write = info.write().await;
                info_write.status = DownloadStatus::Failed(format!("Failed to create file: {}", e));
                return;
            }
        };

        // Stream response body and write to file
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let mut paused = false;
        let check_interval = tokio::time::Duration::from_millis(1);

        loop {
            // Check for control signals with timeout
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
                    // Clean up partial file
                    let _ = tokio::fs::remove_file(&dest_path).await;
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

            // If paused, wait and continue checking for signals
            if paused {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                continue;
            }

            // Try to get next chunk from stream
            match stream.next().await {
                Some(Ok(chunk)) => {
                    // Write chunk to file
                    if let Err(e) = file.write_all(&chunk).await {
                        let mut info_write = info.write().await;
                        info_write.status =
                            DownloadStatus::Failed(format!("Failed to write to file: {}", e));
                        return;
                    }

                    // Update progress
                    downloaded += chunk.len() as u64;
                    {
                        let mut info_write = info.write().await;
                        info_write.downloaded_bytes = downloaded;
                    }
                }
                Some(Err(e)) => {
                    // Network error during streaming
                    let mut info_write = info.write().await;
                    info_write.status = DownloadStatus::Failed(format!("Network error: {}", e));
                    return;
                }
                None => {
                    // Stream ended - download complete
                    break;
                }
            }
        }

        // Flush file to ensure all data is written
        if let Err(e) = file.flush().await {
            let mut info_write = info.write().await;
            info_write.status = DownloadStatus::Failed(format!("Failed to flush file: {}", e));
            return;
        }

        // Mark as complete
        {
            let mut info_write = info.write().await;
            info_write.status = DownloadStatus::Complete;
        }
    }

    /// Mock download task for testing
    ///
    /// This simulates a download without actually fetching from the network.
    /// Used when DOWNLOADS_MOCK_MODE environment variable is set.
    async fn mock_download_task(
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
