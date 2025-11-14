pub mod types;
pub mod validation;
pub mod storage;
pub mod import_export;

pub use types::{Bookmark, BookmarkFolder, BookmarkId};
pub use storage::BookmarkStorage;

use anyhow::Result;

/// Bookmark manager - main API
pub struct BookmarkManager {
    storage: BookmarkStorage,
}

impl BookmarkManager {
    /// Create new bookmark manager with database path
    pub async fn new(db_path: &str) -> Result<Self> {
        let storage = BookmarkStorage::new(db_path)?;
        Ok(Self { storage })
    }

    /// Add new bookmark
    pub async fn add_bookmark(&mut self, mut bookmark: Bookmark) -> Result<BookmarkId> {
        // Generate ID if empty
        if bookmark.id.is_empty() {
            bookmark.id = uuid::Uuid::new_v4().to_string();
        }

        // Set timestamps if not set
        if bookmark.created_at == 0 {
            let now = chrono::Utc::now().timestamp();
            bookmark.created_at = now;
            bookmark.updated_at = now;
        }

        // Validate URL
        validation::validate_url(&bookmark.url)?;

        // Validate folder if present
        if let Some(ref folder) = bookmark.folder {
            validation::validate_folder_path(folder)?;
        }

        // Validate and sanitize
        bookmark.title = validation::sanitize_title(&bookmark.title);

        // Validate tags
        for tag in &bookmark.tags {
            validation::validate_tag(tag)?;
        }

        let id = bookmark.id.clone();
        self.storage.add_bookmark(&bookmark)?;
        Ok(id)
    }

    /// Get bookmark by ID
    pub async fn get_bookmark(&self, id: &BookmarkId) -> Result<Option<Bookmark>> {
        self.storage.get_bookmark(id)
    }

    /// Update existing bookmark
    pub async fn update_bookmark(&mut self, mut bookmark: Bookmark) -> Result<()> {
        // Validate URL
        validation::validate_url(&bookmark.url)?;

        // Validate folder if present
        if let Some(ref folder) = bookmark.folder {
            validation::validate_folder_path(folder)?;
        }

        // Sanitize title
        bookmark.title = validation::sanitize_title(&bookmark.title);

        // Validate tags
        for tag in &bookmark.tags {
            validation::validate_tag(tag)?;
        }

        // Update timestamp
        bookmark.updated_at = chrono::Utc::now().timestamp();

        self.storage.update_bookmark(&bookmark)
    }

    /// Delete bookmark
    pub async fn delete_bookmark(&mut self, id: &BookmarkId) -> Result<()> {
        self.storage.delete_bookmark(id)
    }

    /// List all bookmarks
    pub async fn list_bookmarks(&self) -> Result<Vec<Bookmark>> {
        self.storage.list_bookmarks()
    }

    /// List bookmarks in folder
    pub async fn list_bookmarks_in_folder(&self, folder: &str) -> Result<Vec<Bookmark>> {
        validation::validate_folder_path(folder)?;
        self.storage.list_bookmarks_in_folder(folder)
    }

    /// Search bookmarks by title or URL
    pub async fn search_bookmarks(&self, query: &str) -> Result<Vec<Bookmark>> {
        self.storage.search_bookmarks(query)
    }

    /// Search bookmarks by tag
    pub async fn find_by_tag(&self, tag: &str) -> Result<Vec<Bookmark>> {
        validation::validate_tag(tag)?;
        self.storage.find_by_tag(tag)
    }

    /// Create folder
    pub async fn create_folder(&mut self, path: &str) -> Result<()> {
        validation::validate_folder_path(path)?;
        self.storage.create_folder(path)
    }

    /// Delete folder (and optionally move bookmarks)
    pub async fn delete_folder(&mut self, path: &str, move_to: Option<String>) -> Result<()> {
        validation::validate_folder_path(path)?;

        if let Some(ref target) = move_to {
            validation::validate_folder_path(target)?;

            // Move bookmarks to target folder
            let bookmarks = self.storage.list_bookmarks_in_folder(path)?;
            for mut bookmark in bookmarks {
                bookmark.folder = Some(target.clone());
                bookmark.updated_at = chrono::Utc::now().timestamp();
                self.storage.update_bookmark(&bookmark)?;
            }
        }

        self.storage.delete_folder(path)
    }

    /// List all folders
    pub async fn list_folders(&self) -> Result<Vec<BookmarkFolder>> {
        self.storage.list_folders()
    }

    /// Move bookmark to different folder
    pub async fn move_bookmark(&mut self, id: &BookmarkId, folder: Option<String>) -> Result<()> {
        if let Some(ref folder_path) = folder {
            validation::validate_folder_path(folder_path)?;
        }

        let mut bookmark = self
            .storage
            .get_bookmark(id)?
            .ok_or_else(|| anyhow::anyhow!("Bookmark not found"))?;

        bookmark.folder = folder;
        bookmark.updated_at = chrono::Utc::now().timestamp();
        self.storage.update_bookmark(&bookmark)
    }

    /// Import bookmarks from HTML file
    pub async fn import_html(&mut self, html_content: &str) -> Result<usize> {
        let bookmarks = import_export::parse_html(html_content)?;
        let count = bookmarks.len();

        for bookmark in bookmarks {
            self.storage.add_bookmark(&bookmark)?;
        }

        Ok(count)
    }

    /// Export bookmarks to HTML format
    pub async fn export_html(&self) -> Result<String> {
        let bookmarks = self.storage.list_bookmarks()?;
        import_export::generate_html(&bookmarks)
    }

    /// Get bookmark count
    pub async fn count(&self) -> Result<usize> {
        self.storage.count()
    }

    /// Clear all bookmarks
    pub async fn clear(&mut self) -> Result<()> {
        self.storage.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_manager() -> BookmarkManager {
        BookmarkManager::new(":memory:")
            .await
            .expect("Failed to create test manager")
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = create_test_manager().await;
        assert_eq!(manager.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_add_bookmark() {
        let mut manager = create_test_manager().await;
        let bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );

        let id = manager.add_bookmark(bookmark).await.unwrap();
        assert!(!id.is_empty());
        assert_eq!(manager.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_add_bookmark_generates_id() {
        let mut manager = create_test_manager().await;
        let mut bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );
        bookmark.id = "".to_string(); // Clear ID

        let id = manager.add_bookmark(bookmark).await.unwrap();
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_add_bookmark_validates_url() {
        let mut manager = create_test_manager().await;
        let mut bookmark = Bookmark::new("invalid-url".to_string(), "Example".to_string());
        bookmark.id = "test-id".to_string();

        let result = manager.add_bookmark(bookmark).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_bookmark_sanitizes_title() {
        let mut manager = create_test_manager().await;
        let bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "  Title\nWith\tWhitespace  ".to_string(),
        );

        let id = manager.add_bookmark(bookmark).await.unwrap();
        let retrieved = manager.get_bookmark(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Title With Whitespace");
    }

    #[tokio::test]
    async fn test_get_bookmark() {
        let mut manager = create_test_manager().await;
        let bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );

        let id = manager.add_bookmark(bookmark).await.unwrap();
        let retrieved = manager.get_bookmark(&id).await.unwrap();

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.url, "https://example.com");
        assert_eq!(retrieved.title, "Example");
    }

    #[tokio::test]
    async fn test_update_bookmark() {
        let mut manager = create_test_manager().await;
        let bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );

        let id = manager.add_bookmark(bookmark).await.unwrap();
        let mut bookmark = manager.get_bookmark(&id).await.unwrap().unwrap();

        bookmark.title = "Updated Title".to_string();
        manager.update_bookmark(bookmark).await.unwrap();

        let updated = manager.get_bookmark(&id).await.unwrap().unwrap();
        assert_eq!(updated.title, "Updated Title");
    }

    #[tokio::test]
    async fn test_delete_bookmark() {
        let mut manager = create_test_manager().await;
        let bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );

        let id = manager.add_bookmark(bookmark).await.unwrap();
        manager.delete_bookmark(&id).await.unwrap();

        assert_eq!(manager.count().await.unwrap(), 0);
        assert!(manager.get_bookmark(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_bookmarks() {
        let mut manager = create_test_manager().await;
        manager
            .add_bookmark(Bookmark::new(
                "https://example1.com".to_string(),
                "Example 1".to_string(),
            ))
            .await
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(
                "https://example2.com".to_string(),
                "Example 2".to_string(),
            ))
            .await
            .unwrap();

        let bookmarks = manager.list_bookmarks().await.unwrap();
        assert_eq!(bookmarks.len(), 2);
    }

    #[tokio::test]
    async fn test_search_bookmarks() {
        let mut manager = create_test_manager().await;
        manager
            .add_bookmark(Bookmark::new(
                "https://rust-lang.org".to_string(),
                "Rust Programming".to_string(),
            ))
            .await
            .unwrap();
        manager
            .add_bookmark(Bookmark::new(
                "https://python.org".to_string(),
                "Python".to_string(),
            ))
            .await
            .unwrap();

        let results = manager.search_bookmarks("Rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }

    #[tokio::test]
    async fn test_find_by_tag() {
        let mut manager = create_test_manager().await;
        let mut bookmark = Bookmark::new(
            "https://rust-lang.org".to_string(),
            "Rust".to_string(),
        );
        bookmark.tags = vec!["rust".to_string(), "programming".to_string()];
        manager.add_bookmark(bookmark).await.unwrap();

        let results = manager.find_by_tag("rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://rust-lang.org");
    }

    #[tokio::test]
    async fn test_create_folder() {
        let mut manager = create_test_manager().await;
        manager.create_folder("Programming").await.unwrap();

        let folders = manager.list_folders().await.unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].path, "Programming");
    }

    #[tokio::test]
    async fn test_list_bookmarks_in_folder() {
        let mut manager = create_test_manager().await;
        let mut bookmark = Bookmark::new(
            "https://rust-lang.org".to_string(),
            "Rust".to_string(),
        );
        bookmark.folder = Some("Programming".to_string());
        manager.add_bookmark(bookmark).await.unwrap();

        let bookmarks = manager
            .list_bookmarks_in_folder("Programming")
            .await
            .unwrap();
        assert_eq!(bookmarks.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_folder() {
        let mut manager = create_test_manager().await;
        manager.create_folder("Programming").await.unwrap();
        manager.delete_folder("Programming", None).await.unwrap();

        let folders = manager.list_folders().await.unwrap();
        assert_eq!(folders.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_folder_with_move() {
        let mut manager = create_test_manager().await;
        manager.create_folder("Old").await.unwrap();
        manager.create_folder("New").await.unwrap();

        let mut bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );
        bookmark.folder = Some("Old".to_string());
        manager.add_bookmark(bookmark).await.unwrap();

        manager
            .delete_folder("Old", Some("New".to_string()))
            .await
            .unwrap();

        let bookmarks = manager.list_bookmarks_in_folder("New").await.unwrap();
        assert_eq!(bookmarks.len(), 1);
    }

    #[tokio::test]
    async fn test_move_bookmark() {
        let mut manager = create_test_manager().await;
        let bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );

        let id = manager.add_bookmark(bookmark).await.unwrap();
        manager
            .move_bookmark(&id, Some("Programming".to_string()))
            .await
            .unwrap();

        let bookmark = manager.get_bookmark(&id).await.unwrap().unwrap();
        assert_eq!(bookmark.folder, Some("Programming".to_string()));
    }

    #[tokio::test]
    async fn test_import_html() {
        let mut manager = create_test_manager().await;
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<HTML>
<DL><p>
    <DT><A HREF="https://example.com" ADD_DATE="1234567890">Example</A>
</DL><p>"#;

        let count = manager.import_html(html).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(manager.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_export_html() {
        let mut manager = create_test_manager().await;
        manager
            .add_bookmark(Bookmark::new(
                "https://example.com".to_string(),
                "Example".to_string(),
            ))
            .await
            .unwrap();

        let html = manager.export_html().await.unwrap();
        assert!(html.contains("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
        assert!(html.contains("https://example.com"));
        assert!(html.contains("Example"));
    }

    #[tokio::test]
    async fn test_clear() {
        let mut manager = create_test_manager().await;
        manager
            .add_bookmark(Bookmark::new(
                "https://example.com".to_string(),
                "Example".to_string(),
            ))
            .await
            .unwrap();
        manager.create_folder("Programming").await.unwrap();

        manager.clear().await.unwrap();

        assert_eq!(manager.count().await.unwrap(), 0);
        assert_eq!(manager.list_folders().await.unwrap().len(), 0);
    }
}
