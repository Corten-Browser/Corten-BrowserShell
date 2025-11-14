use downloads::{Download, DownloadId, DownloadStatus};
use downloads::storage::DownloadStorage;
use tempfile::TempDir;

fn create_test_storage() -> (DownloadStorage, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = DownloadStorage::new(db_path.to_str().unwrap()).unwrap();
    (storage, temp_dir)
}

fn create_test_download(id: &str, url: &str, status: DownloadStatus) -> Download {
    Download {
        id: id.to_string(),
        url: url.to_string(),
        file_name: "test.zip".to_string(),
        save_path: "/tmp/test.zip".to_string(),
        mime_type: Some("application/zip".to_string()),
        status,
        created_at: chrono::Utc::now().timestamp(),
        completed_at: None,
    }
}

#[test]
fn test_storage_new_creates_database() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = DownloadStorage::new(db_path.to_str().unwrap());
    assert!(result.is_ok());
    assert!(db_path.exists());
}

#[test]
fn test_storage_insert_download() {
    let (storage, _temp) = create_test_storage();
    let download = create_test_download("test-1", "https://example.com/file.zip", DownloadStatus::Pending);

    let result = storage.insert(&download);
    assert!(result.is_ok());
}

#[test]
fn test_storage_get_download() {
    let (storage, _temp) = create_test_storage();
    let download = create_test_download("test-1", "https://example.com/file.zip", DownloadStatus::Pending);

    storage.insert(&download).unwrap();

    let result = storage.get("test-1");
    assert!(result.is_ok());
    let retrieved = result.unwrap();
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, "test-1");
    assert_eq!(retrieved.url, "https://example.com/file.zip");
}

#[test]
fn test_storage_get_nonexistent() {
    let (storage, _temp) = create_test_storage();

    let result = storage.get("nonexistent");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_storage_update_download() {
    let (storage, _temp) = create_test_storage();
    let mut download = create_test_download("test-1", "https://example.com/file.zip", DownloadStatus::Pending);

    storage.insert(&download).unwrap();

    // Update status
    download.status = DownloadStatus::Downloading {
        bytes_downloaded: 1024,
        bytes_total: Some(2048),
    };

    let result = storage.update(&download);
    assert!(result.is_ok());

    // Verify update
    let retrieved = storage.get("test-1").unwrap().unwrap();
    match retrieved.status {
        DownloadStatus::Downloading { bytes_downloaded, bytes_total } => {
            assert_eq!(bytes_downloaded, 1024);
            assert_eq!(bytes_total, Some(2048));
        }
        _ => panic!("Expected Downloading status"),
    }
}

#[test]
fn test_storage_delete_download() {
    let (storage, _temp) = create_test_storage();
    let download = create_test_download("test-1", "https://example.com/file.zip", DownloadStatus::Pending);

    storage.insert(&download).unwrap();

    let result = storage.delete("test-1");
    assert!(result.is_ok());

    // Verify deletion
    let retrieved = storage.get("test-1").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_storage_list_all() {
    let (storage, _temp) = create_test_storage();

    let download1 = create_test_download("test-1", "https://example.com/file1.zip", DownloadStatus::Pending);
    let download2 = create_test_download("test-2", "https://example.com/file2.zip", DownloadStatus::Completed {
        bytes_downloaded: 1024,
        file_path: "/tmp/file2.zip".to_string(),
    });

    storage.insert(&download1).unwrap();
    storage.insert(&download2).unwrap();

    let result = storage.list_all();
    assert!(result.is_ok());

    let downloads = result.unwrap();
    assert_eq!(downloads.len(), 2);
}

#[test]
fn test_storage_list_by_status() {
    let (storage, _temp) = create_test_storage();

    let download1 = create_test_download("test-1", "https://example.com/file1.zip", DownloadStatus::Pending);
    let download2 = create_test_download("test-2", "https://example.com/file2.zip", DownloadStatus::Completed {
        bytes_downloaded: 1024,
        file_path: "/tmp/file2.zip".to_string(),
    });
    let download3 = create_test_download("test-3", "https://example.com/file3.zip", DownloadStatus::Pending);

    storage.insert(&download1).unwrap();
    storage.insert(&download2).unwrap();
    storage.insert(&download3).unwrap();

    let result = storage.list_by_status_prefix("Pending");
    assert!(result.is_ok());

    let downloads = result.unwrap();
    assert_eq!(downloads.len(), 2);
}

#[test]
fn test_storage_list_completed() {
    let (storage, _temp) = create_test_storage();

    let download1 = create_test_download("test-1", "https://example.com/file1.zip", DownloadStatus::Pending);
    let download2 = create_test_download("test-2", "https://example.com/file2.zip", DownloadStatus::Completed {
        bytes_downloaded: 1024,
        file_path: "/tmp/file2.zip".to_string(),
    });

    storage.insert(&download1).unwrap();
    storage.insert(&download2).unwrap();

    let result = storage.list_by_status_prefix("Completed");
    assert!(result.is_ok());

    let downloads = result.unwrap();
    assert_eq!(downloads.len(), 1);
    assert_eq!(downloads[0].id, "test-2");
}

#[test]
fn test_storage_clear_completed() {
    let (storage, _temp) = create_test_storage();

    let download1 = create_test_download("test-1", "https://example.com/file1.zip", DownloadStatus::Pending);
    let download2 = create_test_download("test-2", "https://example.com/file2.zip", DownloadStatus::Completed {
        bytes_downloaded: 1024,
        file_path: "/tmp/file2.zip".to_string(),
    });
    let download3 = create_test_download("test-3", "https://example.com/file3.zip", DownloadStatus::Completed {
        bytes_downloaded: 2048,
        file_path: "/tmp/file3.zip".to_string(),
    });

    storage.insert(&download1).unwrap();
    storage.insert(&download2).unwrap();
    storage.insert(&download3).unwrap();

    let result = storage.clear_completed();
    assert!(result.is_ok());

    // Verify only completed downloads were deleted
    let all = storage.list_all().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, "test-1");
}
