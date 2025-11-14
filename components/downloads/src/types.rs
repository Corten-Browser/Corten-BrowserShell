use serde::{Deserialize, Serialize};

/// Unique identifier for downloads
pub type DownloadId = String;

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading {
        bytes_downloaded: u64,
        bytes_total: Option<u64>,
    },
    Paused {
        bytes_downloaded: u64,
        bytes_total: Option<u64>,
    },
    Completed {
        bytes_downloaded: u64,
        file_path: String,
    },
    Failed {
        error: String,
    },
    Cancelled,
}

/// Download metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: DownloadId,
    pub url: String,
    pub file_name: String,
    pub save_path: String,
    pub mime_type: Option<String>,
    pub status: DownloadStatus,
    pub created_at: i64, // Unix timestamp
    pub completed_at: Option<i64>,
}

/// Download progress event
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started {
        id: DownloadId,
    },
    Progress {
        id: DownloadId,
        bytes_downloaded: u64,
        bytes_total: Option<u64>,
    },
    Completed {
        id: DownloadId,
        file_path: String,
    },
    Failed {
        id: DownloadId,
        error: String,
    },
    Paused {
        id: DownloadId,
    },
    Resumed {
        id: DownloadId,
    },
    Cancelled {
        id: DownloadId,
    },
}
