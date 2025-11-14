//! Bookmark storage implementation with YAML persistence

use crate::Bookmark;
use shared_types::{BookmarkId, ComponentError};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Manages bookmark storage, retrieval, and search
pub struct BookmarksManager {
    /// In-memory storage of bookmarks
    bookmarks: HashMap<BookmarkId, Bookmark>,

    /// Path to the YAML storage file
    storage_path: PathBuf,
}

impl BookmarksManager {
    /// Create a new BookmarksManager with the specified storage directory
    ///
    /// # Arguments
    ///
    /// * `storage_dir` - Directory where bookmarks.yaml will be stored
    ///
    /// # Returns
    ///
    /// A new BookmarksManager instance
    pub fn new(storage_dir: PathBuf) -> Self {
        let storage_path = storage_dir.join("bookmarks.yaml");
        Self {
            bookmarks: HashMap::new(),
            storage_path,
        }
    }

    /// Load bookmarks from YAML file
    ///
    /// # Arguments
    ///
    /// * `storage_dir` - Directory where bookmarks.yaml is stored
    ///
    /// # Returns
    ///
    /// Result containing loaded BookmarksManager or error
    pub async fn load(storage_dir: PathBuf) -> Result<Self, ComponentError> {
        let storage_path = storage_dir.join("bookmarks.yaml");

        if !storage_path.exists() {
            return Ok(Self {
                bookmarks: HashMap::new(),
                storage_path,
            });
        }

        let contents = fs::read_to_string(&storage_path).await.map_err(|e| {
            ComponentError::InitializationFailed(format!("Failed to read bookmarks file: {}", e))
        })?;

        let bookmarks_vec: Vec<Bookmark> = serde_yaml::from_str(&contents).map_err(|e| {
            ComponentError::InitializationFailed(format!("Failed to parse bookmarks YAML: {}", e))
        })?;

        let mut bookmarks = HashMap::new();
        for bookmark in bookmarks_vec {
            if let Some(id) = bookmark.id {
                bookmarks.insert(id, bookmark);
            }
        }

        Ok(Self {
            bookmarks,
            storage_path,
        })
    }

    /// Add a new bookmark
    ///
    /// # Arguments
    ///
    /// * `bookmark` - Bookmark to add
    ///
    /// # Returns
    ///
    /// Result containing the assigned BookmarkId or error
    pub async fn add_bookmark(
        &mut self,
        mut bookmark: Bookmark,
    ) -> Result<BookmarkId, ComponentError> {
        let id = BookmarkId::new();
        bookmark.id = Some(id);

        self.bookmarks.insert(id, bookmark);
        self.save().await?;

        Ok(id)
    }

    /// Remove a bookmark by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the bookmark to remove
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub async fn remove_bookmark(&mut self, id: BookmarkId) -> Result<(), ComponentError> {
        if self.bookmarks.remove(&id).is_none() {
            return Err(ComponentError::ResourceNotFound(format!(
                "Bookmark with ID {:?} not found",
                id
            )));
        }

        self.save().await?;
        Ok(())
    }

    /// Update an existing bookmark
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the bookmark to update
    /// * `bookmark` - Updated bookmark data
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub async fn update_bookmark(
        &mut self,
        id: BookmarkId,
        mut bookmark: Bookmark,
    ) -> Result<(), ComponentError> {
        if !self.bookmarks.contains_key(&id) {
            return Err(ComponentError::ResourceNotFound(format!(
                "Bookmark with ID {:?} not found",
                id
            )));
        }

        bookmark.id = Some(id);
        self.bookmarks.insert(id, bookmark);
        self.save().await?;

        Ok(())
    }

    /// Get a bookmark by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the bookmark to retrieve
    ///
    /// # Returns
    ///
    /// Option containing the bookmark if found
    pub fn get_bookmark(&self, id: BookmarkId) -> Option<Bookmark> {
        self.bookmarks.get(&id).cloned()
    }

    /// Get all bookmarks
    ///
    /// # Returns
    ///
    /// Vector of all bookmarks
    pub async fn get_all_bookmarks(&self) -> Vec<Bookmark> {
        self.bookmarks.values().cloned().collect()
    }

    /// Search bookmarks by query
    ///
    /// Searches in title, URL, and tags (case-insensitive)
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    ///
    /// # Returns
    ///
    /// Vector of matching bookmarks
    pub async fn search_bookmarks(&self, query: String) -> Vec<Bookmark> {
        let query_lower = query.to_lowercase();

        self.bookmarks
            .values()
            .filter(|bookmark| {
                bookmark.title.to_lowercase().contains(&query_lower)
                    || bookmark.url.to_lowercase().contains(&query_lower)
                    || bookmark
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Save bookmarks to YAML file
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub async fn save(&self) -> Result<(), ComponentError> {
        // Ensure parent directory exists
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ComponentError::InvalidState(format!("Failed to create storage directory: {}", e))
            })?;
        }

        let bookmarks_vec: Vec<Bookmark> = self.bookmarks.values().cloned().collect();

        let yaml = serde_yaml::to_string(&bookmarks_vec).map_err(|e| {
            ComponentError::InvalidState(format!("Failed to serialize bookmarks: {}", e))
        })?;

        let mut file = fs::File::create(&self.storage_path).await.map_err(|e| {
            ComponentError::InvalidState(format!("Failed to create bookmarks file: {}", e))
        })?;

        file.write_all(yaml.as_bytes()).await.map_err(|e| {
            ComponentError::InvalidState(format!("Failed to write bookmarks file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_new_manager_is_empty() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BookmarksManager::new(temp_dir.path().to_path_buf());

        let all = manager.get_all_bookmarks().await;
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_path_buf();

        // Create and save bookmarks
        {
            let mut manager = BookmarksManager::new(storage_path.clone());
            let bookmark = Bookmark::new("https://example.com".to_string(), "Example".to_string());
            manager.add_bookmark(bookmark).await.unwrap();
        }

        // Load in new instance
        {
            let manager = BookmarksManager::load(storage_path).await.unwrap();
            let all = manager.get_all_bookmarks().await;
            assert_eq!(all.len(), 1);
        }
    }
}
