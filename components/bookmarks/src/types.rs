use serde::{Deserialize, Serialize};

/// Unique identifier for bookmarks
pub type BookmarkId = String;

/// Bookmark structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub url: String,
    pub title: String,
    pub folder: Option<String>,  // Path like "Programming/Rust"
    pub tags: Vec<String>,
    pub favicon: Option<Vec<u8>>,
    pub created_at: i64,  // Unix timestamp
    pub updated_at: i64,
}

/// Folder structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookmarkFolder {
    pub path: String,  // "Programming/Rust"
    pub parent: Option<String>,  // "Programming"
    pub children: Vec<String>,  // Subfolders
    pub bookmark_count: usize,
}

impl Bookmark {
    /// Create a new bookmark with generated ID and timestamps
    pub fn new(url: String, title: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            title,
            folder: None,
            tags: Vec::new(),
            favicon: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

impl BookmarkFolder {
    /// Create a new folder
    pub fn new(path: String) -> Self {
        let parent = Self::extract_parent(&path);
        Self {
            path,
            parent,
            children: Vec::new(),
            bookmark_count: 0,
        }
    }

    /// Extract parent folder from path
    fn extract_parent(path: &str) -> Option<String> {
        path.rfind('/').map(|idx| path[..idx].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_new() {
        let bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );

        assert!(!bookmark.id.is_empty());
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.title, "Example");
        assert!(bookmark.folder.is_none());
        assert!(bookmark.tags.is_empty());
        assert!(bookmark.favicon.is_none());
        assert!(bookmark.created_at > 0);
        assert_eq!(bookmark.created_at, bookmark.updated_at);
    }

    #[test]
    fn test_bookmark_touch() {
        let mut bookmark = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
        );
        let original_updated = bookmark.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        bookmark.touch();

        assert!(bookmark.updated_at >= original_updated);
    }

    #[test]
    fn test_folder_new() {
        let folder = BookmarkFolder::new("Programming/Rust".to_string());

        assert_eq!(folder.path, "Programming/Rust");
        assert_eq!(folder.parent, Some("Programming".to_string()));
        assert!(folder.children.is_empty());
        assert_eq!(folder.bookmark_count, 0);
    }

    #[test]
    fn test_folder_new_root() {
        let folder = BookmarkFolder::new("Programming".to_string());

        assert_eq!(folder.path, "Programming");
        assert_eq!(folder.parent, None);
    }

    #[test]
    fn test_folder_extract_parent() {
        assert_eq!(
            BookmarkFolder::extract_parent("Programming/Rust/Tokio"),
            Some("Programming/Rust".to_string())
        );
        assert_eq!(
            BookmarkFolder::extract_parent("Programming"),
            None
        );
    }
}
