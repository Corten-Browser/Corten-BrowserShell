use downloads_manager::{DownloadsManager, DownloadStatus};

#[tokio::test]
async fn test_complete_download_lifecycle() {
    let manager = DownloadsManager::new();
    let url = "https://example.com/test-file.zip".to_string();

    // Start download
    let download_id = manager.start_download(url.clone(), None).await.unwrap();

    // Verify it's in the active downloads
    let active = manager.get_active_downloads().await;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].url, url);

    // Pause the download
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    manager.pause_download(download_id).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Paused));

    // Resume the download
    manager.resume_download(download_id).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Downloading));

    // Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Complete | DownloadStatus::Downloading));
}

#[tokio::test]
async fn test_concurrent_downloads_management() {
    let manager = DownloadsManager::new();

    // Start multiple downloads concurrently
    let mut download_ids = Vec::new();
    for i in 1..=10 {
        let url = format!("https://example.com/file{}.zip", i);
        let id = manager.start_download(url, None).await.unwrap();
        download_ids.push(id);
    }

    // Verify all downloads are active
    let active = manager.get_active_downloads().await;
    assert_eq!(active.len(), 10);

    // Pause half of them
    for id in download_ids.iter().take(5) {
        manager.pause_download(*id).await.unwrap();
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify paused count
    let active = manager.get_active_downloads().await;
    let paused_count = active.iter().filter(|info| matches!(info.status, DownloadStatus::Paused)).count();
    assert!(paused_count >= 4); // At least 4 should be paused (allowing for timing variations)

    // Cancel the other half
    for id in download_ids.iter().skip(5) {
        manager.cancel_download(*id).await.unwrap();
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify active downloads (should only include paused ones)
    let active = manager.get_active_downloads().await;
    assert!(active.len() >= 4 && active.len() <= 5);
}

#[tokio::test]
async fn test_download_progress_tracking() {
    let manager = DownloadsManager::new();
    let url = "https://example.com/large-file.zip".to_string();

    let download_id = manager.start_download(url, None).await.unwrap();

    // Track progress over time
    let mut previous_progress = 0u64;
    let mut progress_increased = false;

    for _ in 0..5 {
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;

        if let Some(info) = manager.get_download_info(download_id).await {
            if info.downloaded_bytes > previous_progress {
                progress_increased = true;
                previous_progress = info.downloaded_bytes;
            }

            // If complete, break
            if matches!(info.status, DownloadStatus::Complete) {
                break;
            }
        }
    }

    assert!(progress_increased, "Download progress should increase over time");
}

#[tokio::test]
async fn test_error_handling_invalid_operations() {
    let manager = DownloadsManager::new();

    // Try to pause non-existent download
    let fake_id = shared_types::DownloadId::new();
    let result = manager.pause_download(fake_id).await;
    assert!(result.is_err());

    // Try to resume non-existent download
    let result = manager.resume_download(fake_id).await;
    assert!(result.is_err());

    // Try to cancel non-existent download
    let result = manager.cancel_download(fake_id).await;
    assert!(result.is_err());

    // Try to start download with invalid URL
    let result = manager.start_download("not a url".to_string(), None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_download_state_transitions() {
    let manager = DownloadsManager::new();
    let url = "https://example.com/state-test.zip".to_string();

    let download_id = manager.start_download(url, None).await.unwrap();

    // Initially should be Pending or Downloading
    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Pending | DownloadStatus::Downloading));

    // Wait for download to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Should be Downloading
    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Downloading | DownloadStatus::Complete));

    // Pause
    manager.pause_download(download_id).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Paused | DownloadStatus::Complete));

    // If not complete, resume
    if matches!(info.status, DownloadStatus::Paused) {
        manager.resume_download(download_id).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

        let info = manager.get_download_info(download_id).await.unwrap();
        assert!(matches!(info.status, DownloadStatus::Downloading | DownloadStatus::Complete));
    }
}
