use downloads_manager::DownloadStatus;

#[test]
fn test_download_status_pending() {
    let status = DownloadStatus::Pending;
    assert!(matches!(status, DownloadStatus::Pending));
}

#[test]
fn test_download_status_downloading() {
    let status = DownloadStatus::Downloading;
    assert!(matches!(status, DownloadStatus::Downloading));
}

#[test]
fn test_download_status_paused() {
    let status = DownloadStatus::Paused;
    assert!(matches!(status, DownloadStatus::Paused));
}

#[test]
fn test_download_status_complete() {
    let status = DownloadStatus::Complete;
    assert!(matches!(status, DownloadStatus::Complete));
}

#[test]
fn test_download_status_failed() {
    let status = DownloadStatus::Failed("Network error".to_string());
    match status {
        DownloadStatus::Failed(msg) => assert_eq!(msg, "Network error"),
        _ => panic!("Expected Failed status"),
    }
}

#[test]
fn test_download_status_cancelled() {
    let status = DownloadStatus::Cancelled;
    assert!(matches!(status, DownloadStatus::Cancelled));
}

#[test]
fn test_download_status_clone() {
    let status1 = DownloadStatus::Downloading;
    let status2 = status1.clone();
    assert!(matches!(status2, DownloadStatus::Downloading));
}

#[test]
fn test_download_status_debug() {
    let status = DownloadStatus::Pending;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Pending"));
}
