//! Unit tests for BookmarksManager

use bookmarks_manager::{Bookmark, BookmarksManager};
use shared_types::BookmarkId;
use tempfile::TempDir;

#[tokio::test]
async fn test_add_bookmark_returns_id() {
    /// Given a BookmarksManager
    /// When adding a bookmark
    /// Then it should return a unique ID
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());

    let result = manager.add_bookmark(bookmark).await;

    assert!(result.is_ok());
    let id = result.unwrap();
    assert!(id.as_uuid() != &uuid::Uuid::nil());
}

#[tokio::test]
async fn test_get_bookmark_by_id() {
    /// Given a bookmark has been added
    /// When retrieving by ID
    /// Then it should return the correct bookmark
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());

    let id = manager.add_bookmark(bookmark.clone()).await.unwrap();
    let retrieved = manager.get_bookmark(id);

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.url, "https://example.com");
    assert_eq!(retrieved.title, "Example");
}

#[tokio::test]
async fn test_get_nonexistent_bookmark_returns_none() {
    /// Given a BookmarksManager with no bookmarks
    /// When retrieving a bookmark by ID
    /// Then it should return None
    let temp_dir = TempDir::new().unwrap();
    let manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let nonexistent_id = BookmarkId::new();
    let result = manager.get_bookmark(nonexistent_id);

    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_all_bookmarks() {
    /// Given multiple bookmarks have been added
    /// When getting all bookmarks
    /// Then it should return all added bookmarks
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark1 = Bookmark::new("https://example1.com".to_string(), "Example 1".to_string());
    let bookmark2 = Bookmark::new("https://example2.com".to_string(), "Example 2".to_string());

    manager.add_bookmark(bookmark1).await.unwrap();
    manager.add_bookmark(bookmark2).await.unwrap();

    let all = manager.get_all_bookmarks().await;

    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_update_bookmark() {
    /// Given a bookmark exists
    /// When updating the bookmark
    /// Then the changes should be persisted
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new(
        "https://example.com".to_string(),
        "Original Title".to_string(),
    );

    let id = manager.add_bookmark(bookmark).await.unwrap();

    let mut updated = manager.get_bookmark(id).unwrap();
    updated.title = "Updated Title".to_string();
    updated.tags = vec!["new-tag".to_string()];

    let result = manager.update_bookmark(id, updated).await;

    assert!(result.is_ok());

    let retrieved = manager.get_bookmark(id).unwrap();
    assert_eq!(retrieved.title, "Updated Title");
    assert_eq!(retrieved.tags.len(), 1);
    assert_eq!(retrieved.tags[0], "new-tag");
}

#[tokio::test]
async fn test_update_nonexistent_bookmark_returns_error() {
    /// Given a bookmark does not exist
    /// When attempting to update it
    /// Then it should return an error
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let nonexistent_id = BookmarkId::new();
    let bookmark = Bookmark::new("https://example.com".to_string(), "Test".to_string());

    let result = manager.update_bookmark(nonexistent_id, bookmark).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_remove_bookmark() {
    /// Given a bookmark exists
    /// When removing the bookmark
    /// Then it should no longer be retrievable
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());

    let id = manager.add_bookmark(bookmark).await.unwrap();

    let result = manager.remove_bookmark(id).await;
    assert!(result.is_ok());

    let retrieved = manager.get_bookmark(id);
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_remove_nonexistent_bookmark_returns_error() {
    /// Given a bookmark does not exist
    /// When attempting to remove it
    /// Then it should return an error
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let nonexistent_id = BookmarkId::new();
    let result = manager.remove_bookmark(nonexistent_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_bookmarks_persist_to_yaml() {
    /// Given bookmarks are added
    /// When saving to YAML
    /// Then they should be loadable in a new instance
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().to_path_buf();

    let bookmark1 = Bookmark::new("https://example.com".to_string(), "Example".to_string());

    // Add bookmark and save
    {
        let mut manager = BookmarksManager::new(storage_path.clone());
        manager.add_bookmark(bookmark1).await.unwrap();
        manager.save().await.unwrap();
    }

    // Load in new instance
    {
        let manager = BookmarksManager::load(storage_path).await.unwrap();
        let all = manager.get_all_bookmarks().await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].url, "https://example.com");
    }
}

#[tokio::test]
async fn test_search_bookmarks_by_title() {
    /// Given bookmarks with different titles
    /// When searching by title query
    /// Then only matching bookmarks should be returned
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark1 = Bookmark::new(
        "https://rust-lang.org".to_string(),
        "Rust Programming Language".to_string(),
    );
    let bookmark2 = Bookmark::new(
        "https://python.org".to_string(),
        "Python Programming".to_string(),
    );
    let bookmark3 = Bookmark::new(
        "https://javascript.com".to_string(),
        "JavaScript Guide".to_string(),
    );

    manager.add_bookmark(bookmark1).await.unwrap();
    manager.add_bookmark(bookmark2).await.unwrap();
    manager.add_bookmark(bookmark3).await.unwrap();

    let results = manager.search_bookmarks("rust".to_string()).await;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming Language");
}

#[tokio::test]
async fn test_search_bookmarks_by_url() {
    /// Given bookmarks with different URLs
    /// When searching by URL query
    /// Then matching bookmarks should be returned
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark1 = Bookmark::new(
        "https://github.com/rust-lang/rust".to_string(),
        "Rust Repository".to_string(),
    );
    let bookmark2 = Bookmark::new("https://docs.rs".to_string(), "Rust Docs".to_string());

    manager.add_bookmark(bookmark1).await.unwrap();
    manager.add_bookmark(bookmark2).await.unwrap();

    let results = manager.search_bookmarks("github".to_string()).await;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url, "https://github.com/rust-lang/rust");
}

#[tokio::test]
async fn test_search_bookmarks_by_tags() {
    /// Given bookmarks with different tags
    /// When searching by tag query
    /// Then bookmarks with matching tags should be returned
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark1 = Bookmark::with_metadata(
        "https://example1.com".to_string(),
        "Example 1".to_string(),
        None,
        vec!["rust".to_string(), "programming".to_string()],
    );
    let bookmark2 = Bookmark::with_metadata(
        "https://example2.com".to_string(),
        "Example 2".to_string(),
        None,
        vec!["python".to_string(), "programming".to_string()],
    );

    manager.add_bookmark(bookmark1).await.unwrap();
    manager.add_bookmark(bookmark2).await.unwrap();

    let results = manager.search_bookmarks("rust".to_string()).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].tags.contains(&"rust".to_string()));
}

#[tokio::test]
async fn test_search_is_case_insensitive() {
    /// Given bookmarks with various cases
    /// When searching with different case
    /// Then results should be case-insensitive
    let temp_dir = TempDir::new().unwrap();
    let mut manager = BookmarksManager::new(temp_dir.path().to_path_buf());

    let bookmark = Bookmark::new(
        "https://EXAMPLE.COM".to_string(),
        "Example Title".to_string(),
    );

    manager.add_bookmark(bookmark).await.unwrap();

    let results1 = manager.search_bookmarks("EXAMPLE".to_string()).await;
    let results2 = manager.search_bookmarks("example".to_string()).await;
    let results3 = manager.search_bookmarks("ExAmPlE".to_string()).await;

    assert_eq!(results1.len(), 1);
    assert_eq!(results2.len(), 1);
    assert_eq!(results3.len(), 1);
}
