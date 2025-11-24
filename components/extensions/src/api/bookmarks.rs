//! Chrome-compatible Bookmarks API (chrome.bookmarks)
//!
//! Provides bookmark query, creation, update, and removal functionality
//! compatible with Chrome's extension bookmarks API.

use crate::api::ExtensionApi;
use crate::permissions::Permission;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Bookmarks API error types
#[derive(Error, Debug)]
pub enum BookmarksApiError {
    #[error("Bookmark not found: {0}")]
    NotFound(String),

    #[error("Permission denied: bookmarks permission required")]
    PermissionDenied,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Cannot modify root folder")]
    CannotModifyRoot,

    #[error("Cannot modify managed bookmark")]
    CannotModifyManaged,

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Bookmark tree node representing a bookmark or folder
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkTreeNode {
    /// Unique identifier for this node
    pub id: String,

    /// ID of the parent folder (None for root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Index within the parent folder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    /// URL of the bookmark (None for folders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Title of the bookmark or folder
    pub title: String,

    /// Creation timestamp (milliseconds since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_added: Option<u64>,

    /// Last modified timestamp for folders (milliseconds since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_group_modified: Option<u64>,

    /// Date last used (milliseconds since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_last_used: Option<u64>,

    /// Indicates an unmodifiable bookmark (managed by policy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmodifiable: Option<UnmodifiableReason>,

    /// Children for folder nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<BookmarkTreeNode>>,
}

/// Reason why a bookmark is unmodifiable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnmodifiableReason {
    /// Managed by enterprise policy
    Managed,
}

/// Details for creating a new bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookmarkDetails {
    /// Parent folder ID (defaults to "Other Bookmarks")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Index within parent (defaults to end)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    /// Title of the bookmark
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// URL of the bookmark (omit to create a folder)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Destination for moving a bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveBookmarkDetails {
    /// New parent folder ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// New index within parent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

/// Changes for updating a bookmark
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookmarkDetails {
    /// New title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// New URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Query object for searching bookmarks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkSearchQuery {
    /// Query string to search in title and URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// Filter by URL (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Filter by title (exact match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Chrome-compatible Bookmarks API
///
/// Provides methods matching chrome.bookmarks API for extension compatibility.
pub struct BookmarksApi {
    /// Callback for getting the bookmark tree
    get_tree_callback: Option<Box<dyn Fn() -> Vec<BookmarkTreeNode> + Send + Sync>>,

    /// Callback for searching bookmarks
    search_callback: Option<Box<dyn Fn(BookmarkSearchQuery) -> Vec<BookmarkTreeNode> + Send + Sync>>,

    /// Maximum depth for getSubTree (reserved for future use)
    #[allow(dead_code)]
    max_sustained_depth: usize,
}

impl BookmarksApi {
    /// Root folder ID for "Bookmarks Bar"
    pub const BOOKMARKS_BAR_ID: &'static str = "1";

    /// Root folder ID for "Other Bookmarks"
    pub const OTHER_BOOKMARKS_ID: &'static str = "2";

    /// Root folder ID for "Mobile Bookmarks"
    pub const MOBILE_BOOKMARKS_ID: &'static str = "3";

    /// Create a new BookmarksApi
    pub fn new() -> Self {
        Self {
            get_tree_callback: None,
            search_callback: None,
            max_sustained_depth: 100,
        }
    }

    /// Set the callback for getting the bookmark tree
    pub fn set_get_tree_callback(
        &mut self,
        callback: Box<dyn Fn() -> Vec<BookmarkTreeNode> + Send + Sync>,
    ) {
        self.get_tree_callback = Some(callback);
    }

    /// Set the callback for searching bookmarks
    pub fn set_search_callback(
        &mut self,
        callback: Box<dyn Fn(BookmarkSearchQuery) -> Vec<BookmarkTreeNode> + Send + Sync>,
    ) {
        self.search_callback = Some(callback);
    }

    /// Get the entire bookmark tree
    ///
    /// # Returns
    ///
    /// Array of root bookmark nodes
    pub fn get_tree(&self) -> Result<Vec<BookmarkTreeNode>, BookmarksApiError> {
        if let Some(ref callback) = self.get_tree_callback {
            Ok(callback())
        } else {
            // Return default empty tree structure
            Ok(vec![BookmarkTreeNode {
                id: "0".to_string(),
                parent_id: None,
                index: None,
                url: None,
                title: String::new(),
                date_added: None,
                date_group_modified: None,
                date_last_used: None,
                unmodifiable: None,
                children: Some(vec![
                    BookmarkTreeNode {
                        id: Self::BOOKMARKS_BAR_ID.to_string(),
                        parent_id: Some("0".to_string()),
                        index: Some(0),
                        url: None,
                        title: "Bookmarks Bar".to_string(),
                        date_added: None,
                        date_group_modified: None,
                        date_last_used: None,
                        unmodifiable: None,
                        children: Some(Vec::new()),
                    },
                    BookmarkTreeNode {
                        id: Self::OTHER_BOOKMARKS_ID.to_string(),
                        parent_id: Some("0".to_string()),
                        index: Some(1),
                        url: None,
                        title: "Other Bookmarks".to_string(),
                        date_added: None,
                        date_group_modified: None,
                        date_last_used: None,
                        unmodifiable: None,
                        children: Some(Vec::new()),
                    },
                ]),
            }])
        }
    }

    /// Get a subtree rooted at the specified node
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the root node
    ///
    /// # Returns
    ///
    /// Array containing the subtree
    pub fn get_sub_tree(&self, id: String) -> Result<Vec<BookmarkTreeNode>, BookmarksApiError> {
        let tree = self.get_tree()?;
        self.find_node_in_tree(&tree, &id)
            .map(|node| vec![node])
            .ok_or_else(|| BookmarksApiError::NotFound(id))
    }

    /// Get a single bookmark node
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the bookmark
    ///
    /// # Returns
    ///
    /// Array containing the bookmark node
    pub fn get(&self, id: String) -> Result<Vec<BookmarkTreeNode>, BookmarksApiError> {
        let tree = self.get_tree()?;
        self.find_node_in_tree(&tree, &id)
            .map(|mut node| {
                // get() returns nodes without children populated
                node.children = None;
                vec![node]
            })
            .ok_or_else(|| BookmarksApiError::NotFound(id))
    }

    /// Get multiple bookmark nodes
    ///
    /// # Arguments
    ///
    /// * `ids` - IDs of the bookmarks
    ///
    /// # Returns
    ///
    /// Array containing the bookmark nodes
    pub fn get_multiple(&self, ids: Vec<String>) -> Result<Vec<BookmarkTreeNode>, BookmarksApiError> {
        let tree = self.get_tree()?;
        let mut results = Vec::new();

        for id in ids {
            if let Some(mut node) = self.find_node_in_tree(&tree, &id) {
                node.children = None;
                results.push(node);
            }
        }

        Ok(results)
    }

    /// Get the children of a folder
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the folder
    ///
    /// # Returns
    ///
    /// Array of child bookmark nodes
    pub fn get_children(&self, id: String) -> Result<Vec<BookmarkTreeNode>, BookmarksApiError> {
        let tree = self.get_tree()?;
        self.find_node_in_tree(&tree, &id)
            .and_then(|node| node.children)
            .ok_or_else(|| BookmarksApiError::NotFound(id))
    }

    /// Get recently added bookmarks
    ///
    /// # Arguments
    ///
    /// * `number_of_items` - Maximum number of items to return
    ///
    /// # Returns
    ///
    /// Array of recently added bookmark nodes
    pub fn get_recent(&self, number_of_items: usize) -> Result<Vec<BookmarkTreeNode>, BookmarksApiError> {
        let tree = self.get_tree()?;
        let mut all_bookmarks = self.flatten_tree(&tree);

        // Filter to actual bookmarks (not folders) and sort by date_added
        all_bookmarks.retain(|b| b.url.is_some());
        all_bookmarks.sort_by(|a, b| {
            b.date_added
                .unwrap_or(0)
                .cmp(&a.date_added.unwrap_or(0))
        });

        all_bookmarks.truncate(number_of_items);
        Ok(all_bookmarks)
    }

    /// Search bookmarks
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    ///
    /// # Returns
    ///
    /// Array of matching bookmark nodes
    pub fn search(&self, query: BookmarkSearchQuery) -> Result<Vec<BookmarkTreeNode>, BookmarksApiError> {
        if let Some(ref callback) = self.search_callback {
            Ok(callback(query))
        } else {
            // Default implementation using tree traversal
            let tree = self.get_tree()?;
            let all_bookmarks = self.flatten_tree(&tree);

            let results = all_bookmarks
                .into_iter()
                .filter(|b| b.url.is_some()) // Only actual bookmarks
                .filter(|b| self.matches_query(b, &query))
                .collect();

            Ok(results)
        }
    }

    /// Create a new bookmark
    ///
    /// # Arguments
    ///
    /// * `details` - Bookmark creation details
    ///
    /// # Returns
    ///
    /// Created bookmark node
    pub fn create(&self, _details: CreateBookmarkDetails) -> Result<BookmarkTreeNode, BookmarksApiError> {
        Err(BookmarksApiError::OperationFailed(
            "Bookmark creation requires integration with BookmarksManager".to_string(),
        ))
    }

    /// Move a bookmark to a new location
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the bookmark to move
    /// * `destination` - New location details
    ///
    /// # Returns
    ///
    /// Updated bookmark node
    pub fn move_bookmark(
        &self,
        id: String,
        _destination: MoveBookmarkDetails,
    ) -> Result<BookmarkTreeNode, BookmarksApiError> {
        // Check if trying to move root folders
        if id == "0" || id == Self::BOOKMARKS_BAR_ID || id == Self::OTHER_BOOKMARKS_ID {
            return Err(BookmarksApiError::CannotModifyRoot);
        }

        Err(BookmarksApiError::OperationFailed(
            "Bookmark move requires integration with BookmarksManager".to_string(),
        ))
    }

    /// Update a bookmark
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the bookmark to update
    /// * `changes` - Changes to apply
    ///
    /// # Returns
    ///
    /// Updated bookmark node
    pub fn update(
        &self,
        id: String,
        _changes: UpdateBookmarkDetails,
    ) -> Result<BookmarkTreeNode, BookmarksApiError> {
        // Check if trying to update root folders
        if id == "0" {
            return Err(BookmarksApiError::CannotModifyRoot);
        }

        Err(BookmarksApiError::OperationFailed(
            "Bookmark update requires integration with BookmarksManager".to_string(),
        ))
    }

    /// Remove a bookmark
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the bookmark to remove
    pub fn remove(&self, id: String) -> Result<(), BookmarksApiError> {
        // Check if trying to remove root folders
        if id == "0" || id == Self::BOOKMARKS_BAR_ID || id == Self::OTHER_BOOKMARKS_ID {
            return Err(BookmarksApiError::CannotModifyRoot);
        }

        Err(BookmarksApiError::OperationFailed(
            "Bookmark removal requires integration with BookmarksManager".to_string(),
        ))
    }

    /// Remove a bookmark tree (folder and all contents)
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the folder to remove
    pub fn remove_tree(&self, id: String) -> Result<(), BookmarksApiError> {
        // Check if trying to remove root folders
        if id == "0" || id == Self::BOOKMARKS_BAR_ID || id == Self::OTHER_BOOKMARKS_ID {
            return Err(BookmarksApiError::CannotModifyRoot);
        }

        Err(BookmarksApiError::OperationFailed(
            "Bookmark tree removal requires integration with BookmarksManager".to_string(),
        ))
    }

    // Helper methods

    fn find_node_in_tree(
        &self,
        nodes: &[BookmarkTreeNode],
        id: &str,
    ) -> Option<BookmarkTreeNode> {
        for node in nodes {
            if node.id == id {
                return Some(node.clone());
            }
            if let Some(ref children) = node.children {
                if let Some(found) = self.find_node_in_tree(children, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn flatten_tree(&self, nodes: &[BookmarkTreeNode]) -> Vec<BookmarkTreeNode> {
        let mut result = Vec::new();
        for node in nodes {
            let mut flat_node = node.clone();
            flat_node.children = None; // Don't include children in flattened list
            result.push(flat_node);

            if let Some(ref children) = node.children {
                result.extend(self.flatten_tree(children));
            }
        }
        result
    }

    fn matches_query(&self, bookmark: &BookmarkTreeNode, query: &BookmarkSearchQuery) -> bool {
        // Match by exact URL
        if let Some(ref url) = query.url {
            if bookmark.url.as_ref() != Some(url) {
                return false;
            }
        }

        // Match by exact title
        if let Some(ref title) = query.title {
            if &bookmark.title != title {
                return false;
            }
        }

        // Match by query string (case-insensitive search in title and URL)
        if let Some(ref query_str) = query.query {
            let query_lower = query_str.to_lowercase();
            let title_match = bookmark.title.to_lowercase().contains(&query_lower);
            let url_match = bookmark
                .url
                .as_ref()
                .map(|u| u.to_lowercase().contains(&query_lower))
                .unwrap_or(false);

            if !title_match && !url_match {
                return false;
            }
        }

        true
    }
}

impl Default for BookmarksApi {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionApi for BookmarksApi {
    fn namespace(&self) -> &str {
        "bookmarks"
    }

    fn required_permission(&self) -> Permission {
        Permission::Bookmarks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::PermissionSet;

    #[test]
    fn test_bookmarks_api_creation() {
        let api = BookmarksApi::new();
        assert_eq!(api.namespace(), "bookmarks");
        assert_eq!(api.required_permission(), Permission::Bookmarks);
    }

    #[test]
    fn test_permission_check() {
        let api = BookmarksApi::new();
        let mut permissions = PermissionSet::new();
        assert!(!api.check_permission(&permissions));

        permissions.add(Permission::Bookmarks);
        assert!(api.check_permission(&permissions));
    }

    #[test]
    fn test_default_tree_structure() {
        let api = BookmarksApi::new();
        let tree = api.get_tree().unwrap();

        assert_eq!(tree.len(), 1);
        let root = &tree[0];
        assert_eq!(root.id, "0");
        assert!(root.children.is_some());

        let children = root.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].id, BookmarksApi::BOOKMARKS_BAR_ID);
        assert_eq!(children[1].id, BookmarksApi::OTHER_BOOKMARKS_ID);
    }

    #[test]
    fn test_cannot_modify_root() {
        let api = BookmarksApi::new();

        let result = api.remove("0".to_string());
        assert!(matches!(result, Err(BookmarksApiError::CannotModifyRoot)));

        let result = api.remove(BookmarksApi::BOOKMARKS_BAR_ID.to_string());
        assert!(matches!(result, Err(BookmarksApiError::CannotModifyRoot)));
    }

    #[test]
    fn test_search_empty_query() {
        let api = BookmarksApi::new();
        let result = api.search(BookmarkSearchQuery::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_bookmark_tree_node_serialization() {
        let node = BookmarkTreeNode {
            id: "123".to_string(),
            parent_id: Some("1".to_string()),
            index: Some(0),
            url: Some("https://example.com".to_string()),
            title: "Example".to_string(),
            date_added: Some(1234567890000),
            date_group_modified: None,
            date_last_used: None,
            unmodifiable: None,
            children: None,
        };

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("id"));
        assert!(json.contains("parentId"));
        assert!(json.contains("url"));
    }

    #[test]
    fn test_create_bookmark_details() {
        let details = CreateBookmarkDetails {
            parent_id: Some(BookmarksApi::OTHER_BOOKMARKS_ID.to_string()),
            index: None,
            title: Some("Test Bookmark".to_string()),
            url: Some("https://test.com".to_string()),
        };

        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("parentId"));
        assert!(json.contains("title"));
        assert!(json.contains("url"));
    }

    #[test]
    fn test_get_with_custom_callback() {
        let mut api = BookmarksApi::new();
        api.set_get_tree_callback(Box::new(|| {
            vec![BookmarkTreeNode {
                id: "0".to_string(),
                parent_id: None,
                index: None,
                url: None,
                title: "Root".to_string(),
                date_added: None,
                date_group_modified: None,
                date_last_used: None,
                unmodifiable: None,
                children: Some(vec![BookmarkTreeNode {
                    id: "custom-1".to_string(),
                    parent_id: Some("0".to_string()),
                    index: Some(0),
                    url: Some("https://custom.com".to_string()),
                    title: "Custom".to_string(),
                    date_added: None,
                    date_group_modified: None,
                    date_last_used: None,
                    unmodifiable: None,
                    children: None,
                }]),
            }]
        }));

        let result = api.get("custom-1".to_string());
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].title, "Custom");
    }
}
