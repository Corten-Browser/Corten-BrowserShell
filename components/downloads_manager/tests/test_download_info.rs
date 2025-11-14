use downloads_manager::{DownloadInfo, DownloadStatus};
use shared_types::DownloadId;

#[test]
fn test_download_info_creation() {
    let id = DownloadId::new();
    let info = DownloadInfo {
        id,
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1024,
        downloaded_bytes: 512,
        status: DownloadStatus::Downloading,
    };

    assert_eq!(info.id, id);
    assert_eq!(info.url, "https://example.com/file.zip");
    assert_eq!(info.destination, "/downloads/file.zip");
    assert_eq!(info.filename, "file.zip");
    assert_eq!(info.total_bytes, 1024);
    assert_eq!(info.downloaded_bytes, 512);
    assert!(matches!(info.status, DownloadStatus::Downloading));
}

#[test]
fn test_download_info_progress_calculation() {
    let id = DownloadId::new();
    let info = DownloadInfo {
        id,
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1000,
        downloaded_bytes: 250,
        status: DownloadStatus::Downloading,
    };

    let progress = (info.downloaded_bytes as f64 / info.total_bytes as f64) * 100.0;
    assert_eq!(progress, 25.0);
}

#[test]
fn test_download_info_zero_bytes() {
    let id = DownloadId::new();
    let info = DownloadInfo {
        id,
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 0,
        downloaded_bytes: 0,
        status: DownloadStatus::Pending,
    };

    assert_eq!(info.total_bytes, 0);
    assert_eq!(info.downloaded_bytes, 0);
}

#[test]
fn test_download_info_clone() {
    let id = DownloadId::new();
    let info1 = DownloadInfo {
        id,
        url: "https://example.com/file.zip".to_string(),
        destination: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: 1024,
        downloaded_bytes: 512,
        status: DownloadStatus::Downloading,
    };

    let info2 = info1.clone();
    assert_eq!(info1.id, info2.id);
    assert_eq!(info1.url, info2.url);
    assert_eq!(info1.downloaded_bytes, info2.downloaded_bytes);
}

#[test]
fn test_download_info_debug() {
    let id = DownloadId::new();
    let info = DownloadInfo {
        id,
        url: "https://example.com/test.txt".to_string(),
        destination: "/downloads/test.txt".to_string(),
        filename: "test.txt".to_string(),
        total_bytes: 100,
        downloaded_bytes: 50,
        status: DownloadStatus::Downloading,
    };

    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("test.txt"));
}
