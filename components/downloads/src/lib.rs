pub mod types;
pub mod validation;
pub mod storage;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

pub use types::*;
use storage::DownloadStorage;

/// Handle for a running download task
struct DownloadHandle {
    task: JoinHandle<()>,
    cancel_tx: mpsc::UnboundedSender<()>,
}

/// Download manager
pub struct DownloadManager {
    storage: DownloadStorage,
    download_dir: PathBuf,
    active_downloads: Arc<RwLock<HashMap<DownloadId, DownloadHandle>>>,
    event_tx: mpsc::UnboundedSender<DownloadEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<DownloadEvent>>>>,
}

impl DownloadManager {
    /// Create new download manager
    pub async fn new(db_path: &str, download_dir: &str) -> Result<Self> {
        let storage = DownloadStorage::new(db_path)?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            storage,
            download_dir: PathBuf::from(download_dir),
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
        })
    }

    /// Start new download
    pub async fn start_download(
        &mut self,
        url: String,
        save_path: Option<String>,
    ) -> Result<DownloadId> {
        // Validate URL
        validation::validate_url(&url)?;

        // Generate download ID
        let id = uuid::Uuid::new_v4().to_string();

        // Extract and sanitize file name
        let file_name = if let Some(path) = &save_path {
            validation::sanitize_file_name(
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("download"),
            )
        } else {
            let extracted = validation::extract_file_name_from_url(&url);
            validation::sanitize_file_name(&extracted)
        };

        // Determine save path
        let final_save_path = if let Some(path) = save_path {
            path
        } else {
            self.download_dir.join(&file_name).to_string_lossy().to_string()
        };

        validation::validate_save_path(&final_save_path)?;

        // Create download record
        let download = Download {
            id: id.clone(),
            url: url.clone(),
            file_name: file_name.clone(),
            save_path: final_save_path.clone(),
            mime_type: None,
            status: DownloadStatus::Pending,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        };

        // Save to storage
        self.storage.insert(&download)?;

        // Start download task
        self.start_download_task(id.clone(), url, final_save_path).await?;

        Ok(id)
    }

    /// Start download task
    async fn start_download_task(
        &mut self,
        id: DownloadId,
        url: String,
        save_path: String,
    ) -> Result<()> {
        let storage = self.storage.clone();
        let event_tx = self.event_tx.clone();
        let active_downloads = self.active_downloads.clone();
        let download_id = id.clone();

        let (cancel_tx, mut cancel_rx) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            // Send started event
            let _ = event_tx.send(DownloadEvent::Started { id: download_id.clone() });

            // Update status to downloading
            if let Ok(Some(mut download)) = storage.get(&download_id) {
                download.status = DownloadStatus::Downloading {
                    bytes_downloaded: 0,
                    bytes_total: Some(1024), // Mock size
                };
                let _ = storage.update(&download);
            }

            // Simulate download with cancellation support
            let mut bytes_downloaded = 0u64;
            let bytes_total = 1024u64;
            let chunk_size = 256u64;

            loop {
                // Check for cancellation
                if cancel_rx.try_recv().is_ok() {
                    // Update status to cancelled
                    if let Ok(Some(mut download)) = storage.get(&download_id) {
                        download.status = DownloadStatus::Cancelled;
                        let _ = storage.update(&download);
                    }

                    let _ = event_tx.send(DownloadEvent::Cancelled {
                        id: download_id.clone(),
                    });

                    // Remove from active downloads
                    active_downloads.write().await.remove(&download_id);
                    return;
                }

                // Simulate download progress
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                bytes_downloaded = (bytes_downloaded + chunk_size).min(bytes_total);

                // Send progress event
                let _ = event_tx.send(DownloadEvent::Progress {
                    id: download_id.clone(),
                    bytes_downloaded,
                    bytes_total: Some(bytes_total),
                });

                // Update status
                if let Ok(Some(mut download)) = storage.get(&download_id) {
                    download.status = DownloadStatus::Downloading {
                        bytes_downloaded,
                        bytes_total: Some(bytes_total),
                    };
                    let _ = storage.update(&download);
                }

                if bytes_downloaded >= bytes_total {
                    break;
                }
            }

            // Write mock file
            if let Err(e) = tokio::fs::write(&save_path, b"mock download content").await {
                // Download failed
                if let Ok(Some(mut download)) = storage.get(&download_id) {
                    download.status = DownloadStatus::Failed {
                        error: e.to_string(),
                    };
                    let _ = storage.update(&download);
                }

                let _ = event_tx.send(DownloadEvent::Failed {
                    id: download_id.clone(),
                    error: e.to_string(),
                });

                active_downloads.write().await.remove(&download_id);
                return;
            }

            // Download completed
            if let Ok(Some(mut download)) = storage.get(&download_id) {
                download.status = DownloadStatus::Completed {
                    bytes_downloaded: bytes_total,
                    file_path: save_path.clone(),
                };
                download.completed_at = Some(chrono::Utc::now().timestamp());
                let _ = storage.update(&download);
            }

            let _ = event_tx.send(DownloadEvent::Completed {
                id: download_id.clone(),
                file_path: save_path,
            });

            // Remove from active downloads
            active_downloads.write().await.remove(&download_id);
        });

        // Store handle
        self.active_downloads.write().await.insert(
            id,
            DownloadHandle {
                task,
                cancel_tx,
            },
        );

        Ok(())
    }

    /// Pause download
    pub async fn pause_download(&mut self, id: &DownloadId) -> Result<()> {
        // For this simplified implementation, pause is similar to cancel but with different status
        let download = self.storage.get(id)?
            .ok_or_else(|| anyhow!("Download not found"))?;

        // Cancel the task
        if let Some(handle) = self.active_downloads.write().await.remove(id) {
            let _ = handle.cancel_tx.send(());
            handle.task.abort();
        }

        // Update status based on current progress
        let mut updated_download = download;
        updated_download.status = match updated_download.status {
            DownloadStatus::Downloading { bytes_downloaded, bytes_total } => {
                DownloadStatus::Paused {
                    bytes_downloaded,
                    bytes_total,
                }
            }
            _ => DownloadStatus::Paused {
                bytes_downloaded: 0,
                bytes_total: None,
            },
        };

        self.storage.update(&updated_download)?;

        let _ = self.event_tx.send(DownloadEvent::Paused { id: id.clone() });

        Ok(())
    }

    /// Resume paused download
    pub async fn resume_download(&mut self, id: &DownloadId) -> Result<()> {
        let download = self.storage.get(id)?
            .ok_or_else(|| anyhow!("Download not found"))?;

        // For simplified implementation, just restart the download
        self.start_download_task(id.clone(), download.url, download.save_path).await?;

        let _ = self.event_tx.send(DownloadEvent::Resumed { id: id.clone() });

        Ok(())
    }

    /// Cancel download
    pub async fn cancel_download(&mut self, id: &DownloadId) -> Result<()> {
        // Send cancel signal to task
        if let Some(handle) = self.active_downloads.write().await.remove(id) {
            let _ = handle.cancel_tx.send(());
            handle.task.abort();
        }

        // Update status
        if let Some(mut download) = self.storage.get(id)? {
            download.status = DownloadStatus::Cancelled;
            self.storage.update(&download)?;

            let _ = self.event_tx.send(DownloadEvent::Cancelled { id: id.clone() });
        }

        Ok(())
    }

    /// Get download by ID
    pub async fn get_download(&self, id: &DownloadId) -> Result<Option<Download>> {
        self.storage.get(id)
    }

    /// List all downloads
    pub async fn list_downloads(&self) -> Result<Vec<Download>> {
        self.storage.list_all()
    }

    /// List active downloads
    pub async fn list_active_downloads(&self) -> Result<Vec<Download>> {
        let active_ids: Vec<String> = self
            .active_downloads
            .read()
            .await
            .keys()
            .cloned()
            .collect();

        let mut active = Vec::new();
        for id in active_ids {
            if let Some(download) = self.storage.get(&id)? {
                active.push(download);
            }
        }

        Ok(active)
    }

    /// List completed downloads
    pub async fn list_completed_downloads(&self) -> Result<Vec<Download>> {
        self.storage.list_by_status_prefix("Completed")
    }

    /// Delete download record
    pub async fn delete_download(&mut self, id: &DownloadId) -> Result<()> {
        // Cancel if active
        if self.active_downloads.read().await.contains_key(id) {
            self.cancel_download(id).await?;
        }

        self.storage.delete(id)
    }

    /// Clear completed downloads
    pub async fn clear_completed(&mut self) -> Result<()> {
        self.storage.clear_completed()
    }

    /// Get event receiver for download events
    pub fn event_receiver(&mut self) -> mpsc::UnboundedReceiver<DownloadEvent> {
        self.event_rx
            .blocking_write()
            .take()
            .expect("Event receiver already taken")
    }

    /// Shutdown manager and cancel all active downloads
    pub async fn shutdown(&mut self) -> Result<()> {
        let active_ids: Vec<String> = self
            .active_downloads
            .read()
            .await
            .keys()
            .cloned()
            .collect();

        for id in active_ids {
            self.cancel_download(&id).await?;
        }

        Ok(())
    }
}
