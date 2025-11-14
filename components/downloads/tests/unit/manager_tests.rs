use downloads::{DownloadManager, DownloadStatus};
use tempfile::TempDir;

async fn create_test_manager() -> (DownloadManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let download_dir = temp_dir.path().join("downloads");
    std::fs::create_dir_all(&download_dir).unwrap();

    let manager = DownloadManager::new(
        db_path.to_str().unwrap(),
        download_dir.to_str().unwrap(),
    )
    .await
    .unwrap();

    (manager, temp_dir)
}

#[tokio::test]
async fn test_manager_new() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let download_dir = temp_dir.path().join("downloads");

    let result = DownloadManager::new(db_path.to_str().unwrap(), download_dir.to_str().unwrap()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_download() {
    let (mut manager, _temp) = create_test_manager().await;

    let result = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await;

    assert!(result.is_ok());
    let id = result.unwrap();
    assert!(!id.is_empty());
}

#[tokio::test]
async fn test_start_download_invalid_url() {
    let (mut manager, _temp) = create_test_manager().await;

    let result = manager.start_download("invalid url".to_string(), None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_download() {
    let (mut manager, _temp) = create_test_manager().await;

    let id = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    let result = manager.get_download(&id).await;
    assert!(result.is_ok());

    let download = result.unwrap();
    assert!(download.is_some());

    let download = download.unwrap();
    assert_eq!(download.id, id);
    assert_eq!(download.url, "https://example.com/file.zip");
}

#[tokio::test]
async fn test_list_downloads() {
    let (mut manager, _temp) = create_test_manager().await;

    manager
        .start_download("https://example.com/file1.zip".to_string(), None)
        .await
        .unwrap();

    manager
        .start_download("https://example.com/file2.zip".to_string(), None)
        .await
        .unwrap();

    let result = manager.list_downloads().await;
    assert!(result.is_ok());

    let downloads = result.unwrap();
    assert_eq!(downloads.len(), 2);
}

#[tokio::test]
async fn test_pause_download() {
    let (mut manager, _temp) = create_test_manager().await;

    let id = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let result = manager.pause_download(&id).await;
    assert!(result.is_ok());

    let download = manager.get_download(&id).await.unwrap().unwrap();
    match download.status {
        DownloadStatus::Paused { .. } => {}
        _ => panic!("Expected Paused status, got {:?}", download.status),
    }
}

#[tokio::test]
async fn test_cancel_download() {
    let (mut manager, _temp) = create_test_manager().await;

    let id = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    let result = manager.cancel_download(&id).await;
    assert!(result.is_ok());

    let download = manager.get_download(&id).await.unwrap().unwrap();
    assert_eq!(download.status, DownloadStatus::Cancelled);
}

#[tokio::test]
async fn test_delete_download() {
    let (mut manager, _temp) = create_test_manager().await;

    let id = manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    let result = manager.delete_download(&id).await;
    assert!(result.is_ok());

    let download = manager.get_download(&id).await.unwrap();
    assert!(download.is_none());
}

#[tokio::test]
async fn test_clear_completed() {
    let (mut manager, _temp) = create_test_manager().await;

    // Start downloads (they will complete quickly with mocked download)
    let id1 = manager
        .start_download("https://example.com/file1.zip".to_string(), None)
        .await
        .unwrap();

    let id2 = manager
        .start_download("https://example.com/file2.zip".to_string(), None)
        .await
        .unwrap();

    // Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Clear completed
    let result = manager.clear_completed().await;
    assert!(result.is_ok());

    // Check that downloads were cleared
    let download1 = manager.get_download(&id1).await.unwrap();
    let download2 = manager.get_download(&id2).await.unwrap();

    // At least one should be cleared (both might be completed)
    assert!(download1.is_none() || download2.is_none());
}

#[tokio::test]
async fn test_shutdown() {
    let (mut manager, _temp) = create_test_manager().await;

    manager
        .start_download("https://example.com/file.zip".to_string(), None)
        .await
        .unwrap();

    let result = manager.shutdown().await;
    assert!(result.is_ok());
}
