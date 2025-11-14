//! Bookmarks Manager Component
//!
//! This component provides bookmark storage, organization, and search functionality
//! for the CortenBrowser Browser Shell.

use serde::{Deserialize, Serialize};
use shared_types::BookmarkId;

mod storage;

pub use storage::BookmarksManager;

/// Represents a bookmark with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique identifier for the bookmark (optional, generated when added)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<BookmarkId>,

    /// URL of the bookmark
    pub url: String,

    /// Title/name of the bookmark
    pub title: String,

    /// Optional folder for organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Timestamp when bookmark was created (Unix timestamp)
    pub created_at: u64,
}

impl Bookmark {
    /// Create a new bookmark
    pub fn new(url: String, title: String) -> Self {
        Self {
            id: None,
            url,
            title,
            folder: None,
            tags: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Create a new bookmark with folder and tags
    pub fn with_metadata(
        url: String,
        title: String,
        folder: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id: None,
            url,
            title,
            folder,
            tags,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bookmark() {
        let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());

        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.title, "Example");
        assert!(bookmark.id.is_none());
        assert!(bookmark.folder.is_none());
        assert!(bookmark.tags.is_empty());
        assert!(bookmark.created_at > 0);
    }

    #[test]
    fn test_bookmark_with_metadata() {
        let bookmark = Bookmark::with_metadata(
            "https://example.com".to_string(),
            "Example".to_string(),
            Some("Work".to_string()),
            vec!["test".to_string()],
        );

        assert_eq!(bookmark.folder, Some("Work".to_string()));
        assert_eq!(bookmark.tags.len(), 1);
    }
}
