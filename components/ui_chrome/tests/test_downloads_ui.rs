//! Tests for downloads UI functionality

use ui_chrome::{DownloadDisplay, DownloadDisplayStatus, UiChrome};
use shared_types::DownloadId;

#[test]
fn test_set_downloads_updates_count() {
    let mut chrome = UiChrome::new();

    let downloads = vec![
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file1.zip".to_string(),
            downloaded_bytes: 500,
            total_bytes: 1000,
            bytes_per_second: 1024,
            eta_seconds: 10,
            status: DownloadDisplayStatus::Downloading,
        },
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file2.zip".to_string(),
            downloaded_bytes: 1000,
            total_bytes: 1000,
            bytes_per_second: 0,
            eta_seconds: 0,
            status: DownloadDisplayStatus::Complete,
        },
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file3.zip".to_string(),
            downloaded_bytes: 200,
            total_bytes: 1000,
            bytes_per_second: 0,
            eta_seconds: 0,
            status: DownloadDisplayStatus::Paused,
        },
    ];

    chrome.set_downloads(downloads.clone());

    // Should count Downloading and Paused, not Complete
    assert_eq!(chrome.get_download_count(), 2);
    assert_eq!(chrome.get_downloads().len(), 3);
}

#[test]
fn test_get_downloads_returns_all() {
    let mut chrome = UiChrome::new();

    let downloads = vec![
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file1.zip".to_string(),
            downloaded_bytes: 500,
            total_bytes: 1000,
            bytes_per_second: 1024,
            eta_seconds: 10,
            status: DownloadDisplayStatus::Downloading,
        },
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file2.zip".to_string(),
            downloaded_bytes: 1000,
            total_bytes: 1000,
            bytes_per_second: 0,
            eta_seconds: 0,
            status: DownloadDisplayStatus::Complete,
        },
    ];

    chrome.set_downloads(downloads.clone());

    let retrieved = chrome.get_downloads();
    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved[0].filename, "file1.zip");
    assert_eq!(retrieved[1].filename, "file2.zip");
}

#[test]
fn test_clear_completed_downloads() {
    let mut chrome = UiChrome::new();

    let downloads = vec![
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file1.zip".to_string(),
            downloaded_bytes: 500,
            total_bytes: 1000,
            bytes_per_second: 1024,
            eta_seconds: 10,
            status: DownloadDisplayStatus::Downloading,
        },
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file2.zip".to_string(),
            downloaded_bytes: 1000,
            total_bytes: 1000,
            bytes_per_second: 0,
            eta_seconds: 0,
            status: DownloadDisplayStatus::Complete,
        },
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file3.zip".to_string(),
            downloaded_bytes: 1000,
            total_bytes: 1000,
            bytes_per_second: 0,
            eta_seconds: 0,
            status: DownloadDisplayStatus::Complete,
        },
    ];

    chrome.set_downloads(downloads);
    assert_eq!(chrome.get_downloads().len(), 3);

    chrome.clear_completed_downloads();

    // Only downloading should remain
    assert_eq!(chrome.get_downloads().len(), 1);
    assert_eq!(chrome.get_downloads()[0].filename, "file1.zip");
    assert_eq!(chrome.get_download_count(), 1);
}

#[test]
fn test_clear_completed_downloads_empty() {
    let mut chrome = UiChrome::new();

    let downloads = vec![DownloadDisplay {
        id: DownloadId::new(),
        filename: "file1.zip".to_string(),
        downloaded_bytes: 1000,
        total_bytes: 1000,
        bytes_per_second: 0,
        eta_seconds: 0,
        status: DownloadDisplayStatus::Complete,
    }];

    chrome.set_downloads(downloads);
    assert_eq!(chrome.get_downloads().len(), 1);

    chrome.clear_completed_downloads();

    // All downloads cleared
    assert_eq!(chrome.get_downloads().len(), 0);
    assert_eq!(chrome.get_download_count(), 0);
}

#[test]
fn test_download_count_with_failed_downloads() {
    let mut chrome = UiChrome::new();

    let downloads = vec![
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file1.zip".to_string(),
            downloaded_bytes: 500,
            total_bytes: 1000,
            bytes_per_second: 1024,
            eta_seconds: 10,
            status: DownloadDisplayStatus::Downloading,
        },
        DownloadDisplay {
            id: DownloadId::new(),
            filename: "file2.zip".to_string(),
            downloaded_bytes: 100,
            total_bytes: 1000,
            bytes_per_second: 0,
            eta_seconds: 0,
            status: DownloadDisplayStatus::Failed("Network error".to_string()),
        },
    ];

    chrome.set_downloads(downloads);

    // Failed downloads should not count as active
    assert_eq!(chrome.get_download_count(), 1);
    assert_eq!(chrome.get_downloads().len(), 2);
}

#[test]
fn test_download_display_status_equality() {
    assert_eq!(
        DownloadDisplayStatus::Downloading,
        DownloadDisplayStatus::Downloading
    );
    assert_eq!(DownloadDisplayStatus::Paused, DownloadDisplayStatus::Paused);
    assert_eq!(
        DownloadDisplayStatus::Complete,
        DownloadDisplayStatus::Complete
    );

    assert_ne!(
        DownloadDisplayStatus::Downloading,
        DownloadDisplayStatus::Paused
    );
}

#[test]
fn test_empty_downloads_list() {
    let chrome = UiChrome::new();

    assert_eq!(chrome.get_downloads().len(), 0);
    assert_eq!(chrome.get_download_count(), 0);
}
