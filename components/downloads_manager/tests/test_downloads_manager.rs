use downloads_manager::{DownloadStatus, DownloadsManager};
use shared_types::DownloadId;

#[tokio::test]
async fn test_downloads_manager_creation() {
    let manager = DownloadsManager::new();
    assert!(manager.get_active_downloads().await.is_empty());
}

#[tokio::test]
async fn test_start_download_creates_download() {
    let manager = DownloadsManager::new();
    let url = "https://example.com/file.zip".to_string();
    let destination = Some("/downloads/file.zip".to_string());

    let result = manager.start_download(url.clone(), destination).await;
    assert!(result.is_ok());

    let download_id = result.unwrap();
    let info = manager.get_download_info(download_id).await;
    assert!(info.is_some());

    let info = info.unwrap();
    assert_eq!(info.url, url);
    assert_eq!(info.filename, "file.zip");
}

#[tokio::test]
async fn test_start_download_with_default_destination() {
    let manager = DownloadsManager::new();
    let url = "https://example.com/document.pdf".to_string();

    let result = manager.start_download(url.clone(), None).await;
    assert!(result.is_ok());

    let download_id = result.unwrap();
    let info = manager.get_download_info(download_id).await;
    assert!(info.is_some());

    let info = info.unwrap();
    assert_eq!(info.filename, "document.pdf");
    assert!(info.destination.contains("document.pdf"));
}

#[tokio::test]
async fn test_start_download_invalid_url() {
    let manager = DownloadsManager::new();
    let url = "not-a-valid-url".to_string();

    let result = manager.start_download(url, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_pause_download() {
    // Use mock manager for predictable testing (avoids env var race conditions)
    let manager = DownloadsManager::new_mock();
    let url = "https://example.com/file.zip".to_string();

    let download_id = manager.start_download(url, None).await.unwrap();

    // Give download time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let result = manager.pause_download(download_id).await;
    assert!(result.is_ok());

    // Give time for pause signal to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Paused));
}

#[tokio::test]
async fn test_pause_nonexistent_download() {
    let manager = DownloadsManager::new();
    let fake_id = DownloadId::new();

    let result = manager.pause_download(fake_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resume_download() {
    // Use mock manager for predictable testing (avoids env var race conditions)
    let manager = DownloadsManager::new_mock();
    let url = "https://example.com/file.zip".to_string();

    let download_id = manager.start_download(url, None).await.unwrap();

    // Give download time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Pause first
    manager.pause_download(download_id).await.unwrap();

    // Give time for pause signal to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    // Then resume
    let result = manager.resume_download(download_id).await;
    assert!(result.is_ok());

    // Give time for resume signal to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Downloading));
}

#[tokio::test]
async fn test_resume_nonexistent_download() {
    let manager = DownloadsManager::new();
    let fake_id = DownloadId::new();

    let result = manager.resume_download(fake_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cancel_download() {
    // Use mock manager for predictable testing (avoids env var race conditions)
    let manager = DownloadsManager::new_mock();
    let url = "https://example.com/large-file.zip".to_string();

    let download_id = manager.start_download(url, None).await.unwrap();

    // Give download time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let result = manager.cancel_download(download_id).await;
    assert!(result.is_ok());

    // Give time for cancel signal to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let info = manager.get_download_info(download_id).await.unwrap();
    assert!(matches!(info.status, DownloadStatus::Cancelled));
}

#[tokio::test]
async fn test_cancel_nonexistent_download() {
    let manager = DownloadsManager::new();
    let fake_id = DownloadId::new();

    let result = manager.cancel_download(fake_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_download_info_nonexistent() {
    let manager = DownloadsManager::new();
    let fake_id = DownloadId::new();

    let info = manager.get_download_info(fake_id).await;
    assert!(info.is_none());
}

#[tokio::test]
async fn test_get_active_downloads() {
    let manager = DownloadsManager::new();

    // Start multiple downloads
    let url1 = "https://example.com/file1.zip".to_string();
    let url2 = "https://example.com/file2.pdf".to_string();
    let url3 = "https://example.com/file3.txt".to_string();

    manager.start_download(url1, None).await.unwrap();
    manager.start_download(url2, None).await.unwrap();
    manager.start_download(url3, None).await.unwrap();

    let active = manager.get_active_downloads().await;
    assert_eq!(active.len(), 3);
}

#[tokio::test]
async fn test_get_active_downloads_excludes_cancelled() {
    // Use mock manager for predictable testing (avoids env var race conditions)
    let manager = DownloadsManager::new_mock();

    let url1 = "https://example.com/file1.zip".to_string();
    let url2 = "https://example.com/file2.pdf".to_string();

    let id1 = manager.start_download(url1, None).await.unwrap();
    manager.start_download(url2, None).await.unwrap();

    // Cancel one download
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    manager.cancel_download(id1).await.unwrap();

    // Give time for cancel signal to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let active = manager.get_active_downloads().await;
    assert_eq!(active.len(), 1);
}

#[tokio::test]
async fn test_multiple_concurrent_downloads() {
    let manager = DownloadsManager::new();

    let urls = vec![
        "https://example.com/file1.zip",
        "https://example.com/file2.pdf",
        "https://example.com/file3.txt",
        "https://example.com/file4.doc",
        "https://example.com/file5.mp3",
    ];

    for url in urls {
        manager.start_download(url.to_string(), None).await.unwrap();
    }

    let active = manager.get_active_downloads().await;
    assert_eq!(active.len(), 5);
}

#[tokio::test]
async fn test_download_progress_increases() {
    // Use mock manager for predictable testing (avoids env var race conditions)
    let manager = DownloadsManager::new_mock();
    let url = "https://example.com/file.zip".to_string();

    let download_id = manager.start_download(url, None).await.unwrap();

    // Initial progress should be 0
    let info1 = manager.get_download_info(download_id).await.unwrap();
    let progress1 = info1.downloaded_bytes;

    // Wait a bit for progress
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Progress should have increased
    let info2 = manager.get_download_info(download_id).await.unwrap();
    let progress2 = info2.downloaded_bytes;

    assert!(progress2 > progress1 || matches!(info2.status, DownloadStatus::Complete));
}
