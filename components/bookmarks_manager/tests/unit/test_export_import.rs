//! Unit tests for export/import functionality

use bookmarks_manager::{Bookmark, BookmarksManager};
use tempfile::TempDir;

#[tokio::test]
async fn test_export_to_json_creates_file() {
    /// Given a BookmarksManager with bookmarks
    /// When exporting to JSON
    /// Then a JSON file should be created
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
    manager.add_bookmark(bookmark).await.unwrap();

    let export_path = temp_dir.path().join("export.json");
    let result = manager.export_to_json(&export_path).await;

    assert!(result.is_ok());
    assert!(export_path.exists());
}

#[tokio::test]
async fn test_export_to_json_includes_metadata() {
    /// Given a BookmarksManager with bookmarks
    /// When exporting to JSON
    /// Then the JSON should include metadata (version, timestamp, count)
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
    manager.add_bookmark(bookmark).await.unwrap();

    let export_path = temp_dir.path().join("export.json");
    manager.export_to_json(&export_path).await.unwrap();

    // Read and verify JSON structure
    let contents = tokio::fs::read_to_string(&export_path).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert!(json.get("version").is_some());
    assert!(json.get("exported_at").is_some());
    assert!(json.get("bookmark_count").is_some());
    assert!(json.get("bookmarks").is_some());
    assert_eq!(json["bookmark_count"], 1);
}

#[tokio::test]
async fn test_export_to_json_is_pretty_printed() {
    /// Given a BookmarksManager with bookmarks
    /// When exporting to JSON
    /// Then the JSON should be pretty-printed for readability
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
    manager.add_bookmark(bookmark).await.unwrap();

    let export_path = temp_dir.path().join("export.json");
    manager.export_to_json(&export_path).await.unwrap();

    let contents = tokio::fs::read_to_string(&export_path).await.unwrap();

    // Pretty-printed JSON should have newlines and indentation
    assert!(contents.contains('\n'));
    assert!(contents.contains("  "));
}

#[tokio::test]
async fn test_import_from_json_loads_bookmarks() {
    /// Given a JSON export file with bookmarks
    /// When importing from JSON
    /// Then all bookmarks should be loaded
    let temp_dir = TempDir::new().unwrap();

    // Create and export bookmarks
    let export_path = temp_dir.path().join("export.json");
    {
        let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());
        let bookmark1 = Bookmark::new("https://example1.com".to_string(), "Example 1".to_string());
        let bookmark2 = Bookmark::new("https://example2.com".to_string(), "Example 2".to_string());
        manager.add_bookmark(bookmark1).await.unwrap();
        manager.add_bookmark(bookmark2).await.unwrap();
        manager.export_to_json(&export_path).await.unwrap();
    }

    // Import into new manager
    let mut new_manager = BookmarksManager::new(temp_dir.path().join("new"));
    let count = new_manager.import_from_json(&export_path).await.unwrap();

    assert_eq!(count, 2);
    assert_eq!(new_manager.get_all_bookmarks().await.len(), 2);
}

#[tokio::test]
async fn test_import_from_json_returns_imported_count() {
    /// Given a JSON export file with bookmarks
    /// When importing from JSON
    /// Then it should return the count of imported bookmarks
    let temp_dir = TempDir::new().unwrap();

    let export_path = temp_dir.path().join("export.json");
    {
        let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());
        let bookmark1 = Bookmark::new("https://example1.com".to_string(), "Example 1".to_string());
        let bookmark2 = Bookmark::new("https://example2.com".to_string(), "Example 2".to_string());
        let bookmark3 = Bookmark::new("https://example3.com".to_string(), "Example 3".to_string());
        manager.add_bookmark(bookmark1).await.unwrap();
        manager.add_bookmark(bookmark2).await.unwrap();
        manager.add_bookmark(bookmark3).await.unwrap();
        manager.export_to_json(&export_path).await.unwrap();
    }

    let mut new_manager = BookmarksManager::new(temp_dir.path().join("new"));
    let count = new_manager.import_from_json(&export_path).await.unwrap();

    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_import_from_json_merges_with_existing() {
    /// Given a manager with existing bookmarks
    /// When importing from JSON
    /// Then it should merge with existing bookmarks (no duplicates by URL)
    let temp_dir = TempDir::new().unwrap();

    // Create export with bookmarks
    let export_path = temp_dir.path().join("export.json");
    {
        let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());
        let bookmark1 = Bookmark::new("https://example1.com".to_string(), "Example 1".to_string());
        let bookmark2 = Bookmark::new("https://example2.com".to_string(), "Example 2".to_string());
        manager.add_bookmark(bookmark1).await.unwrap();
        manager.add_bookmark(bookmark2).await.unwrap();
        manager.export_to_json(&export_path).await.unwrap();
    }

    // Import into manager that already has one of the same bookmarks
    let mut new_manager = BookmarksManager::new(temp_dir.path().join("new"));
    let existing = Bookmark::new("https://example1.com".to_string(), "Existing".to_string());
    new_manager.add_bookmark(existing).await.unwrap();

    let count = new_manager.import_from_json(&export_path).await.unwrap();

    // Should import only 1 new bookmark (example2), not the duplicate (example1)
    assert_eq!(count, 1);
    assert_eq!(new_manager.get_all_bookmarks().await.len(), 2);
}

#[tokio::test]
async fn test_import_from_json_validates_structure() {
    /// Given an invalid JSON file
    /// When attempting to import
    /// Then it should return an error
    let temp_dir = TempDir::new().unwrap();
    let invalid_json_path = temp_dir.path().join("invalid.json");

    // Write invalid JSON
    tokio::fs::write(&invalid_json_path, b"{ invalid json }")
        .await
        .unwrap();

    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());
    let result = manager.import_from_json(&invalid_json_path).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_backup_bookmarks_creates_timestamped_file() {
    /// Given a BookmarksManager with bookmarks
    /// When creating a backup
    /// Then a timestamped backup file should be created
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
    manager.add_bookmark(bookmark).await.unwrap();

    let backup_path = manager.backup_bookmarks().await.unwrap();

    assert!(backup_path.exists());
    let filename = backup_path.file_name().unwrap().to_str().unwrap();
    assert!(filename.starts_with("bookmarks_backup_"));
    assert!(filename.ends_with(".json"));
}

#[tokio::test]
async fn test_backup_bookmarks_returns_path() {
    /// Given a BookmarksManager with bookmarks
    /// When creating a backup
    /// Then it should return the path to the backup file
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
    manager.add_bookmark(bookmark).await.unwrap();

    let backup_path = manager.backup_bookmarks().await.unwrap();

    assert!(backup_path.is_absolute() || backup_path.exists());
}

#[tokio::test]
async fn test_backup_bookmarks_preserves_all_data() {
    /// Given a BookmarksManager with multiple bookmarks
    /// When creating a backup
    /// Then all bookmark data should be preserved
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark1 = Bookmark::with_metadata(
        "https://example1.com".to_string(),
        "Example 1".to_string(),
        Some("Work".to_string()),
        vec!["tag1".to_string()],
    );
    let bookmark2 = Bookmark::new("https://example2.com".to_string(), "Example 2".to_string());
    manager.add_bookmark(bookmark1).await.unwrap();
    manager.add_bookmark(bookmark2).await.unwrap();

    let backup_path = manager.backup_bookmarks().await.unwrap();

    // Verify backup can be imported
    let mut restore_manager = BookmarksManager::new(temp_dir.path().join("restore"));
    let count = restore_manager
        .import_from_json(&backup_path)
        .await
        .unwrap();

    assert_eq!(count, 2);
    let restored = restore_manager.get_all_bookmarks().await;
    assert_eq!(restored.len(), 2);

    // Verify metadata is preserved
    let with_folder = restored.iter().find(|b| b.folder.is_some()).unwrap();
    assert_eq!(with_folder.folder, Some("Work".to_string()));
    assert_eq!(with_folder.tags.len(), 1);
}

#[tokio::test]
async fn test_export_import_roundtrip() {
    /// Given bookmarks with complete metadata
    /// When exporting and then importing
    /// Then all data should be preserved
    let temp_dir = TempDir::new().unwrap();

    let export_path = temp_dir.path().join("roundtrip.json");

    // Create bookmarks with full metadata
    let original_bookmarks = {
        let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());
        let bookmark1 = Bookmark::with_metadata(
            "https://example1.com".to_string(),
            "Example 1".to_string(),
            Some("Work".to_string()),
            vec!["rust".to_string(), "programming".to_string()],
        );
        let bookmark2 = Bookmark::with_metadata(
            "https://example2.com".to_string(),
            "Example 2".to_string(),
            Some("Personal".to_string()),
            vec!["blog".to_string()],
        );
        manager.add_bookmark(bookmark1).await.unwrap();
        manager.add_bookmark(bookmark2).await.unwrap();
        manager.export_to_json(&export_path).await.unwrap();
        manager.get_all_bookmarks().await
    };

    // Import into new manager
    let mut import_manager = BookmarksManager::new(temp_dir.path().join("import"));
    import_manager.import_from_json(&export_path).await.unwrap();
    let imported_bookmarks = import_manager.get_all_bookmarks().await;

    assert_eq!(imported_bookmarks.len(), original_bookmarks.len());

    for original in &original_bookmarks {
        let imported = imported_bookmarks
            .iter()
            .find(|b| b.url == original.url)
            .expect("Bookmark should be imported");

        assert_eq!(imported.title, original.title);
        assert_eq!(imported.folder, original.folder);
        assert_eq!(imported.tags, original.tags);
        assert_eq!(imported.created_at, original.created_at);
    }
}
