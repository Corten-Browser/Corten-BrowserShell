//! Chrome-compatible Tabs API (chrome.tabs)
//!
//! Provides tab query, creation, update, and removal functionality
//! compatible with Chrome's extension tabs API.

use crate::api::ExtensionApi;
use crate::permissions::Permission;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Tabs API error types
#[derive(Error, Debug)]
pub enum TabsApiError {
    #[error("Tab not found: {0}")]
    NotFound(u64),

    #[error("Permission denied: tabs permission required")]
    PermissionDenied,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Tab information returned by the API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTabInfo {
    /// Tab ID
    pub id: u64,

    /// Index in the window
    pub index: usize,

    /// Window containing this tab
    pub window_id: u64,

    /// Whether this tab is active in its window
    pub active: bool,

    /// Whether this tab is pinned
    pub pinned: bool,

    /// Whether this tab is highlighted
    pub highlighted: bool,

    /// Whether this tab is in incognito mode
    pub incognito: bool,

    /// Tab URL (requires tabs permission)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Tab title (requires tabs permission)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Tab favicon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fav_icon_url: Option<String>,

    /// Tab loading status
    pub status: TabStatus,

    /// Whether the tab has audio playing
    pub audible: bool,

    /// Whether the tab is muted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted_info: Option<MutedInfo>,

    /// Whether the tab is discarded
    pub discarded: bool,

    /// Whether the tab can be discarded
    pub auto_discardable: bool,

    /// Group ID if the tab is in a group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i64>,
}

/// Tab loading status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TabStatus {
    /// Tab is loading
    Loading,
    /// Tab is fully loaded
    Complete,
    /// Tab content is unloaded (suspended)
    Unloaded,
}

/// Muted information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutedInfo {
    /// Whether the tab is muted
    pub muted: bool,

    /// Reason for muting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<MutedReason>,

    /// ID of extension that muted the tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_id: Option<String>,
}

/// Reason for muting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MutedReason {
    /// Muted by user
    User,
    /// Muted by tab capture
    Capture,
    /// Muted by extension
    Extension,
}

/// Query parameters for finding tabs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabQuery {
    /// Filter by active state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Filter by pinned state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,

    /// Filter by audible state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audible: Option<bool>,

    /// Filter by muted state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,

    /// Filter by highlighted state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted: Option<bool>,

    /// Filter by discarded state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discarded: Option<bool>,

    /// Filter by auto-discardable state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_discardable: Option<bool>,

    /// Filter by current window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_window: Option<bool>,

    /// Filter by last focused window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_focused_window: Option<bool>,

    /// Filter by loading status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TabStatus>,

    /// Filter by title (pattern matching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Filter by URL (pattern matching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Vec<String>>,

    /// Filter by window ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<u64>,

    /// Filter by window type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_type: Option<String>,

    /// Filter by index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    /// Filter by group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i64>,
}

/// Properties for creating a new tab
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTabProperties {
    /// Window to create the tab in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<u64>,

    /// Index to insert the tab at
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    /// URL to navigate to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Whether to make the tab active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Whether to pin the tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,

    /// ID of the opener tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opener_tab_id: Option<u64>,
}

/// Properties for updating a tab
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTabProperties {
    /// URL to navigate to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Whether to make the tab active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Whether to highlight the tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted: Option<bool>,

    /// Whether to pin the tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,

    /// Whether to mute the tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,

    /// ID of the opener tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opener_tab_id: Option<u64>,

    /// Whether to auto-discard the tab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_discardable: Option<bool>,
}

/// Move properties for relocating a tab
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveTabProperties {
    /// Destination window ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<u64>,

    /// New index position
    pub index: i32,
}

/// Chrome-compatible Tabs API
///
/// Provides methods matching chrome.tabs API for extension compatibility.
pub struct TabsApi {
    /// Callback for querying tabs (injected from tab manager)
    query_callback: Option<Box<dyn Fn(TabQuery) -> Vec<QueryTabInfo> + Send + Sync>>,
}

impl TabsApi {
    /// Create a new TabsApi
    pub fn new() -> Self {
        Self {
            query_callback: None,
        }
    }

    /// Set the query callback (used by integration with tab manager)
    pub fn set_query_callback(
        &mut self,
        callback: Box<dyn Fn(TabQuery) -> Vec<QueryTabInfo> + Send + Sync>,
    ) {
        self.query_callback = Some(callback);
    }

    /// Query for tabs matching the given criteria
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for filtering tabs
    ///
    /// # Returns
    ///
    /// Vector of tabs matching the query
    pub fn query(&self, query: TabQuery) -> Result<Vec<QueryTabInfo>, TabsApiError> {
        if let Some(ref callback) = self.query_callback {
            Ok(callback(query))
        } else {
            // Return empty list if no callback is set (for testing)
            Ok(Vec::new())
        }
    }

    /// Get a specific tab by ID
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to get
    ///
    /// # Returns
    ///
    /// Tab information if found
    pub fn get(&self, tab_id: u64) -> Result<QueryTabInfo, TabsApiError> {
        let query = TabQuery::default();
        let tabs = self.query(query)?;
        tabs.into_iter()
            .find(|t| t.id == tab_id)
            .ok_or(TabsApiError::NotFound(tab_id))
    }

    /// Create a new tab
    ///
    /// # Arguments
    ///
    /// * `props` - Properties for the new tab
    ///
    /// # Returns
    ///
    /// Created tab information
    pub fn create(&self, _props: CreateTabProperties) -> Result<QueryTabInfo, TabsApiError> {
        // This would integrate with the tab manager
        // For now, return a placeholder error
        Err(TabsApiError::OperationFailed(
            "Tab creation requires integration with TabManager".to_string(),
        ))
    }

    /// Update an existing tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to update
    /// * `props` - Properties to update
    ///
    /// # Returns
    ///
    /// Updated tab information
    pub fn update(
        &self,
        _tab_id: u64,
        _props: UpdateTabProperties,
    ) -> Result<QueryTabInfo, TabsApiError> {
        // This would integrate with the tab manager
        Err(TabsApiError::OperationFailed(
            "Tab update requires integration with TabManager".to_string(),
        ))
    }

    /// Remove one or more tabs
    ///
    /// # Arguments
    ///
    /// * `tab_ids` - IDs of tabs to remove
    ///
    /// # Returns
    ///
    /// Success or error
    pub fn remove(&self, _tab_ids: Vec<u64>) -> Result<(), TabsApiError> {
        // This would integrate with the tab manager
        Err(TabsApiError::OperationFailed(
            "Tab removal requires integration with TabManager".to_string(),
        ))
    }

    /// Move tabs to a new position
    ///
    /// # Arguments
    ///
    /// * `tab_ids` - IDs of tabs to move
    /// * `props` - Move destination properties
    ///
    /// # Returns
    ///
    /// Moved tab information
    pub fn move_tabs(
        &self,
        _tab_ids: Vec<u64>,
        _props: MoveTabProperties,
    ) -> Result<Vec<QueryTabInfo>, TabsApiError> {
        Err(TabsApiError::OperationFailed(
            "Tab move requires integration with TabManager".to_string(),
        ))
    }

    /// Reload a tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to reload
    /// * `bypass_cache` - Whether to bypass the cache
    ///
    /// # Returns
    ///
    /// Success or error
    pub fn reload(&self, _tab_id: Option<u64>, _bypass_cache: bool) -> Result<(), TabsApiError> {
        Err(TabsApiError::OperationFailed(
            "Tab reload requires integration with TabManager".to_string(),
        ))
    }

    /// Duplicate a tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to duplicate
    ///
    /// # Returns
    ///
    /// Duplicated tab information
    pub fn duplicate(&self, _tab_id: u64) -> Result<QueryTabInfo, TabsApiError> {
        Err(TabsApiError::OperationFailed(
            "Tab duplication requires integration with TabManager".to_string(),
        ))
    }

    /// Get the currently active tab in the specified window
    ///
    /// # Arguments
    ///
    /// * `window_id` - Window to get active tab from (None for current window)
    ///
    /// # Returns
    ///
    /// Active tab information
    pub fn get_current(&self, window_id: Option<u64>) -> Result<QueryTabInfo, TabsApiError> {
        let mut query = TabQuery::default();
        query.active = Some(true);
        if let Some(wid) = window_id {
            query.window_id = Some(wid);
        } else {
            query.current_window = Some(true);
        }

        let tabs = self.query(query)?;
        tabs.into_iter()
            .next()
            .ok_or(TabsApiError::OperationFailed("No active tab found".to_string()))
    }

    /// Discard a tab to free memory
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to discard
    ///
    /// # Returns
    ///
    /// Discarded tab information
    pub fn discard(&self, _tab_id: Option<u64>) -> Result<QueryTabInfo, TabsApiError> {
        Err(TabsApiError::OperationFailed(
            "Tab discard requires integration with TabManager".to_string(),
        ))
    }

    /// Go back in the tab's navigation history
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab (None for current tab)
    pub fn go_back(&self, _tab_id: Option<u64>) -> Result<(), TabsApiError> {
        Err(TabsApiError::OperationFailed(
            "Navigation requires integration with TabManager".to_string(),
        ))
    }

    /// Go forward in the tab's navigation history
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab (None for current tab)
    pub fn go_forward(&self, _tab_id: Option<u64>) -> Result<(), TabsApiError> {
        Err(TabsApiError::OperationFailed(
            "Navigation requires integration with TabManager".to_string(),
        ))
    }
}

impl Default for TabsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionApi for TabsApi {
    fn namespace(&self) -> &str {
        "tabs"
    }

    fn required_permission(&self) -> Permission {
        Permission::Tabs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::PermissionSet;

    #[test]
    fn test_tabs_api_creation() {
        let api = TabsApi::new();
        assert_eq!(api.namespace(), "tabs");
        assert_eq!(api.required_permission(), Permission::Tabs);
    }

    #[test]
    fn test_query_with_callback() {
        let mut api = TabsApi::new();
        api.set_query_callback(Box::new(|_query| {
            vec![QueryTabInfo {
                id: 1,
                index: 0,
                window_id: 1,
                active: true,
                pinned: false,
                highlighted: true,
                incognito: false,
                url: Some("https://example.com".to_string()),
                title: Some("Example".to_string()),
                fav_icon_url: None,
                status: TabStatus::Complete,
                audible: false,
                muted_info: None,
                discarded: false,
                auto_discardable: true,
                group_id: None,
            }]
        }));

        let result = api.query(TabQuery::default());
        assert!(result.is_ok());
        let tabs = result.unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].id, 1);
    }

    #[test]
    fn test_query_without_callback() {
        let api = TabsApi::new();
        let result = api.query(TabQuery::default());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_permission_check() {
        let api = TabsApi::new();
        let mut permissions = PermissionSet::new();
        assert!(!api.check_permission(&permissions));

        permissions.add(Permission::Tabs);
        assert!(api.check_permission(&permissions));
    }

    #[test]
    fn test_tab_query_serialization() {
        let query = TabQuery {
            active: Some(true),
            pinned: Some(false),
            ..Default::default()
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("active"));
        assert!(json.contains("pinned"));
    }

    #[test]
    fn test_create_tab_properties() {
        let props = CreateTabProperties {
            url: Some("https://example.com".to_string()),
            active: Some(true),
            ..Default::default()
        };
        assert!(props.url.is_some());
        assert_eq!(props.active, Some(true));
    }
}
