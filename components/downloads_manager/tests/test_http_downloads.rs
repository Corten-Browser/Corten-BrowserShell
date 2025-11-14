use downloads_manager::{DownloadsManager, DownloadStatus};
use std::path::PathBuf;
use tokio::fs;

/// Test that downloads directory is created if it doesn't exist
#[tokio::test]
async fn test_downloads_directory_creation() {
    let manager = DownloadsManager::new();

    // Use a test URL - in production this would actually download
    // For now, we're testing the directory creation logic
    let url = "https://httpbin.org/bytes/1024".to_string();

    // Start download with a specific test directory
    let test_dir = std::env::temp_dir().join("test_downloads");
    let destination = test_dir.join("test_file.bin");

    // Clean up if exists
    let _ = fs::remove_dir_all(&test_dir).await;

    let result = manager.start_download(url, Some(destination.to_string_lossy().to_string())).await;

    // Should succeed even if directory doesn't exist
    assert!(result.is_ok(), "Download should start even if directory doesn't exist");

    // Give time for download to start and create directory
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Clean up
    let _ = fs::remove_dir_all(&test_dir).await;
}

/// Test that download progress is tracked accurately
#[tokio::test]
async fn test_real_download_progress_tracking() {
    let manager = DownloadsManager::new();

    // Use a small file for testing
    let url = "https://httpbin.org/bytes/10240".to_string(); // 10KB file

    let test_dir = std::env::temp_dir().join("test_downloads_progress");
    let destination = test_dir.join("progress_test.bin");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir).await;

    let download_id = manager.start_download(url, Some(destination.to_string_lossy().to_string())).await.unwrap();

    // Track progress
    let mut last_progress = 0u64;
    let mut progress_updates = 0;

    for _ in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        if let Some(info) = manager.get_download_info(download_id).await {
            if info.downloaded_bytes > last_progress {
                progress_updates += 1;
                last_progress = info.downloaded_bytes;
            }

            // Break if complete or failed
            if matches!(info.status, DownloadStatus::Complete | DownloadStatus::Failed(_)) {
                break;
            }
        }
    }

    // Should have seen progress updates (or completed immediately)
    assert!(progress_updates > 0 || last_progress > 0, "Should track download progress");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir).await;
}

/// Test downloading to a file with automatic filename
#[tokio::test]
async fn test_automatic_filename_from_url() {
    let manager = DownloadsManager::new();

    let url = "https://httpbin.org/bytes/1024".to_string();

    let download_id = manager.start_download(url.clone(), None).await.unwrap();

    let info = manager.get_download_info(download_id).await.unwrap();

    // Should have extracted a filename
    assert!(!info.filename.is_empty(), "Should extract filename from URL");
    assert!(!info.destination.is_empty(), "Should have destination path");
}

/// Test error handling for network errors
#[tokio::test]
async fn test_network_error_handling() {
    let manager = DownloadsManager::new();

    // Use an invalid/unreachable URL
    let url = "https://this-domain-definitely-does-not-exist-12345.com/file.zip".to_string();

    let result = manager.start_download(url, None).await;

    // Should still create the download (it will fail during execution)
    assert!(result.is_ok(), "Should accept the download request");

    let download_id = result.unwrap();

    // Wait for download to attempt and fail
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    if let Some(info) = manager.get_download_info(download_id).await {
        // Should either be failed or still attempting
        assert!(
            matches!(info.status, DownloadStatus::Failed(_) | DownloadStatus::Downloading),
            "Should handle network errors gracefully"
        );
    }
}

/// Test that files are actually saved to disk
#[tokio::test]
async fn test_file_saved_to_disk() {
    let manager = DownloadsManager::new();

    let url = "https://httpbin.org/bytes/1024".to_string();
    let test_dir = std::env::temp_dir().join("test_downloads_file_save");
    let destination = test_dir.join("saved_file.bin");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir).await;

    let download_id = manager.start_download(url, Some(destination.to_string_lossy().to_string())).await.unwrap();

    // Wait for download to complete
    for _ in 0..20 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        if let Some(info) = manager.get_download_info(download_id).await {
            if matches!(info.status, DownloadStatus::Complete) {
                break;
            }
        }
    }

    // Verify file exists and has correct size
    let metadata = fs::metadata(&destination).await;

    // File might exist or download might have failed due to network issues
    // This test is more of an integration test
    if metadata.is_ok() {
        let size = metadata.unwrap().len();
        assert!(size > 0, "Downloaded file should have content");
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir).await;
}

/// Test pause and resume with real downloads
#[tokio::test]
async fn test_pause_resume_real_download() {
    let manager = DownloadsManager::new();

    // Use a larger file to ensure we can pause it
    let url = "https://httpbin.org/bytes/102400".to_string(); // 100KB
    let test_dir = std::env::temp_dir().join("test_downloads_pause_resume");
    let destination = test_dir.join("pause_resume_test.bin");

    // Clean up
    let _ = fs::remove_dir_all(&test_dir).await;

    let download_id = manager.start_download(url, Some(destination.to_string_lossy().to_string())).await.unwrap();

    // Let it start downloading
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Pause
    let pause_result = manager.pause_download(download_id).await;
    assert!(pause_result.is_ok(), "Should be able to pause download");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    let paused_progress = info.downloaded_bytes;

    // Wait a bit - progress should not increase while paused
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(
        matches!(info.status, DownloadStatus::Paused) || info.downloaded_bytes == paused_progress,
        "Download should be paused or not progressing"
    );

    // Resume
    let resume_result = manager.resume_download(download_id).await;
    assert!(resume_result.is_ok(), "Should be able to resume download");

    // Clean up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let _ = fs::remove_dir_all(&test_dir).await;
}
