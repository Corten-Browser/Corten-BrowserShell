//! Tests for download metrics and progress calculation

use downloads_manager::{DownloadInfo, DownloadStatus};
use shared_types::DownloadId;

#[test]
fn test_progress_fraction_with_known_size() {
    let info = DownloadInfo {
        id: DownloadId::new(),
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1000,
        downloaded_bytes: 500,
        status: DownloadStatus::Downloading,
    };

    assert_eq!(info.progress_fraction(), 0.5);
}

#[test]
fn test_progress_fraction_complete() {
    let info = DownloadInfo {
        id: DownloadId::new(),
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1000,
        downloaded_bytes: 1000,
        status: DownloadStatus::Complete,
    };

    assert_eq!(info.progress_fraction(), 1.0);
}

#[test]
fn test_progress_fraction_unknown_size() {
    let info = DownloadInfo {
        id: DownloadId::new(),
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 0,
        downloaded_bytes: 500,
        status: DownloadStatus::Downloading,
    };

    assert_eq!(info.progress_fraction(), 0.0);
}

#[test]
fn test_progress_percentage() {
    let info = DownloadInfo {
        id: DownloadId::new(),
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1000,
        downloaded_bytes: 750,
        status: DownloadStatus::Downloading,
    };

    assert_eq!(info.progress_percentage(), 75);
}

#[test]
fn test_progress_percentage_zero() {
    let info = DownloadInfo {
        id: DownloadId::new(),
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1000,
        downloaded_bytes: 0,
        status: DownloadStatus::Pending,
    };

    assert_eq!(info.progress_percentage(), 0);
}

#[test]
fn test_progress_percentage_complete() {
    let info = DownloadInfo {
        id: DownloadId::new(),
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1000,
        downloaded_bytes: 1000,
        status: DownloadStatus::Complete,
    };

    assert_eq!(info.progress_percentage(), 100);
}

#[tokio::test]
async fn test_get_all_downloads() {
    // Set mock mode for testing
    std::env::set_var("DOWNLOADS_MOCK_MODE", "1");

    let manager = downloads_manager::DownloadsManager::new();

    // Start some downloads
    let id1 = manager
        .start_download("https://example.com/file1.zip".to_string(), None)
        .await
        .unwrap();
    let id2 = manager
        .start_download("https://example.com/file2.zip".to_string(), None)
        .await
        .unwrap();

    // Get all downloads
    let all_downloads = manager.get_all_downloads().await;

    assert_eq!(all_downloads.len(), 2);
    assert!(all_downloads.iter().any(|d| d.id == id1));
    assert!(all_downloads.iter().any(|d| d.id == id2));

    // Cleanup
    std::env::remove_var("DOWNLOADS_MOCK_MODE");
}

#[tokio::test]
async fn test_get_all_download_metrics() {
    // Set mock mode for testing
    std::env::set_var("DOWNLOADS_MOCK_MODE", "1");

    let manager = downloads_manager::DownloadsManager::new();

    // Start a download
    let _id = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    // Wait longer for download to progress (200ms should be enough for mock download)
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Get metrics
    let metrics = manager.get_all_download_metrics().await;

    assert_eq!(metrics.len(), 1);
    assert!(
        metrics[0].info.downloaded_bytes > 0,
        "Download should have progressed, got {} bytes",
        metrics[0].info.downloaded_bytes
    );
    // Speed should be calculated for active downloads
    assert!(
        metrics[0].bytes_per_second > 0,
        "Speed should be > 0, got {}",
        metrics[0].bytes_per_second
    );

    // Cleanup
    std::env::remove_var("DOWNLOADS_MOCK_MODE");
}

#[tokio::test]
async fn test_get_download_metrics() {
    // Set mock mode for testing
    std::env::set_var("DOWNLOADS_MOCK_MODE", "1");

    let manager = downloads_manager::DownloadsManager::new();

    // Start a download
    let id = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    // Wait longer for download to progress (200ms should be enough for mock download)
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Get metrics for specific download
    let metrics = manager.get_download_metrics(id).await;

    assert!(metrics.is_some());
    let metrics = metrics.unwrap();
    assert_eq!(metrics.info.id, id);
    assert!(
        metrics.info.downloaded_bytes > 0,
        "Download should have progressed, got {} bytes",
        metrics.info.downloaded_bytes
    );
    assert!(
        metrics.bytes_per_second > 0,
        "Speed should be > 0, got {}",
        metrics.bytes_per_second
    );

    // Cleanup
    std::env::remove_var("DOWNLOADS_MOCK_MODE");
}

#[tokio::test]
async fn test_metrics_for_paused_download() {
    // Set mock mode for testing
    std::env::set_var("DOWNLOADS_MOCK_MODE", "1");

    let manager = downloads_manager::DownloadsManager::new();

    // Start a download
    let id = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    // Wait longer for download to start (200ms should be enough for mock download)
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    manager.pause_download(id).await.unwrap();

    // Wait for pause to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Get metrics for paused download
    let metrics = manager.get_download_metrics(id).await.unwrap();

    // Status should be paused
    assert_eq!(metrics.info.status, DownloadStatus::Paused);

    // Speed should be 0 for paused downloads
    assert_eq!(metrics.bytes_per_second, 0);

    // Cleanup
    std::env::remove_var("DOWNLOADS_MOCK_MODE");
}
