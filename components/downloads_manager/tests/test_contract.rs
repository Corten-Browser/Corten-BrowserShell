//! Contract tests for downloads_manager
//!
//! These tests verify that the DownloadsManager component implements
//! the exact API defined in contracts/downloads_manager.yaml

use downloads_manager::{DownloadInfo, DownloadStatus, DownloadsManager};
use shared_types::{ComponentError, DownloadId};

#[test]
fn test_download_status_variants() {
    // Verify all required variants exist
    let _pending = DownloadStatus::Pending;
    let _downloading = DownloadStatus::Downloading;
    let _paused = DownloadStatus::Paused;
    let _complete = DownloadStatus::Complete;
    let _failed = DownloadStatus::Failed("test".to_string());
    let _cancelled = DownloadStatus::Cancelled;
}

#[test]
fn test_download_info_fields() {
    // Verify DownloadInfo has all required fields
    let info = DownloadInfo {
        id: DownloadId::new(),
        url: String::new(),
        destination: String::new(),
        filename: String::new(),
        total_bytes: 0,
        downloaded_bytes: 0,
        status: DownloadStatus::Pending,
    };

    // Access each field to verify they exist
    let _id = info.id;
    let _url = info.url;
    let _destination = info.destination;
    let _filename = info.filename;
    let _total_bytes = info.total_bytes;
    let _downloaded_bytes = info.downloaded_bytes;
    let _status = info.status;
}

#[tokio::test]
async fn test_downloads_manager_start_download_signature() {
    let manager = DownloadsManager::new();

    // Verify start_download has correct signature:
    // async fn start_download(url: String, destination: Option<String>) -> Result<DownloadId, ComponentError>
    let result: Result<DownloadId, ComponentError> = manager
        .start_download(
            "https://example.com/file.zip".to_string(),
            Some("/downloads/file.zip".to_string()),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_downloads_manager_pause_download_signature() {
    let manager = DownloadsManager::new();
    let id = DownloadId::new();

    // Verify pause_download has correct signature:
    // async fn pause_download(id: DownloadId) -> Result<(), ComponentError>
    let result: Result<(), ComponentError> = manager.pause_download(id).await;

    // Should error because download doesn't exist, but signature is correct
    assert!(result.is_err());
}

#[tokio::test]
async fn test_downloads_manager_resume_download_signature() {
    let manager = DownloadsManager::new();
    let id = DownloadId::new();

    // Verify resume_download has correct signature:
    // async fn resume_download(id: DownloadId) -> Result<(), ComponentError>
    let result: Result<(), ComponentError> = manager.resume_download(id).await;

    // Should error because download doesn't exist, but signature is correct
    assert!(result.is_err());
}

#[tokio::test]
async fn test_downloads_manager_cancel_download_signature() {
    let manager = DownloadsManager::new();
    let id = DownloadId::new();

    // Verify cancel_download has correct signature:
    // async fn cancel_download(id: DownloadId) -> Result<(), ComponentError>
    let result: Result<(), ComponentError> = manager.cancel_download(id).await;

    // Should error because download doesn't exist, but signature is correct
    assert!(result.is_err());
}

#[tokio::test]
async fn test_downloads_manager_get_download_info_signature() {
    let manager = DownloadsManager::new();
    let id = DownloadId::new();

    // Verify get_download_info has correct signature:
    // fn get_download_info(id: DownloadId) -> Option<DownloadInfo>
    let result: Option<DownloadInfo> = manager.get_download_info(id).await;

    assert!(result.is_none());
}

#[tokio::test]
async fn test_downloads_manager_get_active_downloads_signature() {
    let manager = DownloadsManager::new();

    // Verify get_active_downloads has correct signature:
    // fn get_active_downloads() -> Vec<DownloadInfo>
    let result: Vec<DownloadInfo> = manager.get_active_downloads().await;

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_contract_start_download_returns_download_id() {
    let manager = DownloadsManager::new();

    let result = manager
        .start_download("https://example.com/test.zip".to_string(), None)
        .await;

    assert!(result.is_ok());
    let download_id = result.unwrap();

    // Verify it's a valid DownloadId by using it
    let info = manager.get_download_info(download_id).await;
    assert!(info.is_some());
}

#[tokio::test]
async fn test_contract_error_handling() {
    let manager = DownloadsManager::new();

    // Invalid URL should return ComponentError
    let result = manager.start_download("not a url".to_string(), None).await;

    assert!(result.is_err());
    match result {
        Err(ComponentError::InvalidState(_)) => {
            // Correct error type
        }
        _ => panic!("Expected ComponentError::InvalidState"),
    }
}

#[tokio::test]
async fn test_contract_download_info_contains_all_data() {
    let manager = DownloadsManager::new();
    let url = "https://example.com/document.pdf".to_string();

    let download_id = manager.start_download(url.clone(), None).await.unwrap();

    let info = manager.get_download_info(download_id).await.unwrap();

    // Verify all contract fields are populated
    assert_eq!(info.id, download_id);
    assert_eq!(info.url, url);
    assert!(!info.destination.is_empty());
    assert_eq!(info.filename, "document.pdf");
    assert!(info.total_bytes > 0);
    assert!(info.downloaded_bytes >= 0);
    assert!(matches!(
        info.status,
        DownloadStatus::Pending | DownloadStatus::Downloading
    ));
}
