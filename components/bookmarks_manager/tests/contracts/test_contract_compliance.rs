//! Contract compliance tests
//!
//! Ensures that the bookmarks_manager component implements the exact API
//! defined in contracts/bookmarks_manager.yaml

use bookmarks_manager::{Bookmark, BookmarksManager};
use shared_types::{BookmarkId, ComponentError};
use tempfile::TempDir;

#[test]
fn test_bookmark_struct_has_required_fields() {
    /// Contract specifies Bookmark with specific fields
    /// Verify all required fields exist
    let bookmark = Bookmark {
        id: Some(BookmarkId::new()),
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        folder: Some("Work".to_string()),
        tags: vec!["test".to_string()],
        created_at: 1234567890,
    };

    // Verify fields are accessible per contract
    assert!(bookmark.id.is_some());
    assert!(!bookmark.url.is_empty());
    assert!(!bookmark.title.is_empty());
    assert!(bookmark.folder.is_some());
    assert!(!bookmark.tags.is_empty());
    assert!(bookmark.created_at > 0);
}

#[tokio::test]
async fn test_add_bookmark_signature() {
    /// Contract: add_bookmark(bookmark: Bookmark) -> Result<BookmarkId, ComponentError>
    /// Verify method exists with correct signature
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());

    // Contract specifies this returns Result<BookmarkId, ComponentError>
    let result: Result<BookmarkId, ComponentError> = manager.add_bookmark(bookmark).await;

    assert!(result.is_ok());
    assert!(result.unwrap().as_uuid() != &uuid::Uuid::nil());
}

#[tokio::test]
async fn test_remove_bookmark_signature() {
    /// Contract: remove_bookmark(id: BookmarkId) -> Result<(), ComponentError>
    /// Verify method exists with correct signature
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
    let id = manager.add_bookmark(bookmark).await.unwrap();

    // Contract specifies this returns Result<(), ComponentError>
    let result: Result<(), ComponentError> = manager.remove_bookmark(id).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_bookmark_signature() {
    /// Contract: update_bookmark(id: BookmarkId, bookmark: Bookmark) -> Result<(), ComponentError>
    /// Verify method exists with correct signature
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
    let id = manager.add_bookmark(bookmark.clone()).await.unwrap();

    // Contract specifies this returns Result<(), ComponentError>
    let result: Result<(), ComponentError> = manager.update_bookmark(id, bookmark).await;

    assert!(result.is_ok());
}

#[test]
fn test_get_bookmark_signature() {
    /// Contract: get_bookmark(id: BookmarkId) -> Option<Bookmark>
    /// Verify method exists with correct signature (not async)
    let temp_dir = TempDir::new().unwrap();
    let manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let id = BookmarkId::new();

    // Contract specifies this is NOT async and returns Option<Bookmark>
    let result: Option<Bookmark> = manager.get_bookmark(id);

    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_all_bookmarks_signature() {
    /// Contract: get_all_bookmarks() -> Vec<Bookmark>
    /// Verify method exists with correct signature
    let temp_dir = TempDir::new().unwrap();
    let manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    // Contract specifies this returns Vec<Bookmark>
    let result: Vec<Bookmark> = manager.get_all_bookmarks().await;

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_search_bookmarks_signature() {
    /// Contract: search_bookmarks(query: String) -> Vec<Bookmark>
    /// Verify method exists with correct signature
    let temp_dir = TempDir::new().unwrap();
    let manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    // Contract specifies this takes String and returns Vec<Bookmark>
    let result: Vec<Bookmark> = manager.search_bookmarks("test".to_string()).await;

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_error_types_match_contract() {
    /// Contract specifies ComponentError for all async operations
    /// Verify errors are of correct type
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let nonexistent_id = BookmarkId::new();

    // Test remove_bookmark error type
    let remove_error = manager.remove_bookmark(nonexistent_id).await;
    assert!(remove_error.is_err());
    let _: ComponentError = remove_error.unwrap_err(); // Verify it's ComponentError

    // Test update_bookmark error type
    let bookmark = Bookmark::new("https://example.com".to_string(), "Test".to_string());
    let update_error = manager.update_bookmark(nonexistent_id, bookmark).await;
    assert!(update_error.is_err());
    let _: ComponentError = update_error.unwrap_err(); // Verify it's ComponentError
}

#[test]
fn test_bookmark_optional_fields_are_optional() {
    /// Contract specifies id and folder as Option types
    /// Verify they can be None
    let bookmark = Bookmark {
        id: None,
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        folder: None,
        tags: vec![],
        created_at: 1234567890,
    };

    assert!(bookmark.id.is_none());
    assert!(bookmark.folder.is_none());
}

#[test]
fn test_bookmark_id_type() {
    /// Contract specifies id as BookmarkId type
    /// Verify type is correct
    let id = BookmarkId::new();
    let bookmark = Bookmark {
        id: Some(id),
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        folder: None,
        tags: vec![],
        created_at: 1234567890,
    };

    // Verify type matches contract
    let _: Option<BookmarkId> = bookmark.id;
}

#[test]
fn test_bookmark_tags_is_vec_string() {
    /// Contract specifies tags as Vec<String>
    /// Verify type is correct
    let bookmark = Bookmark {
        id: None,
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        folder: None,
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        created_at: 1234567890,
    };

    // Verify type matches contract
    let _: Vec<String> = bookmark.tags;
}

#[test]
fn test_bookmark_created_at_is_u64() {
    /// Contract specifies created_at as u64
    /// Verify type is correct
    let bookmark = Bookmark {
        id: None,
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        folder: None,
        tags: vec![],
        created_at: 1234567890,
    };

    // Verify type matches contract
    let _: u64 = bookmark.created_at;
}
