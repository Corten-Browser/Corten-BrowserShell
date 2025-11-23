//! Tab Manager Component
//!
//! This component manages tab lifecycle, navigation, and history tracking
//! for the CortenBrowser Browser Shell.
//!
//! Supports private/incognito browsing mode where no data is persisted.

use shared_types::{ProcessId, RenderSurfaceId, TabError, TabId, WindowId};
use std::collections::HashMap;
use url::Url;

/// Tab representation containing all tab state
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    pub window_id: WindowId,
    pub title: String,
    pub url: Option<Url>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub favicon: Option<Vec<u8>>,
    pub process_id: Option<ProcessId>,
    pub render_surface: RenderSurfaceId,
    /// Whether this tab is in private/incognito mode.
    /// Private tabs do not persist history, cookies, or cache.
    pub is_private: bool,
}

/// Navigation history entry
#[derive(Debug, Clone)]
struct HistoryEntry {
    url: Url,
    title: String,
}

/// Navigation history for a tab
#[derive(Debug, Clone)]
struct NavigationHistory {
    entries: Vec<HistoryEntry>,
    current_index: Option<usize>,
}

impl NavigationHistory {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
        }
    }

    fn push(&mut self, url: Url, title: String) {
        // If we're not at the end of history, truncate forward entries
        if let Some(current) = self.current_index {
            self.entries.truncate(current + 1);
        }

        self.entries.push(HistoryEntry { url, title });
        self.current_index = Some(self.entries.len() - 1);
    }

    fn can_go_back(&self) -> bool {
        if let Some(current) = self.current_index {
            current > 0
        } else {
            false
        }
    }

    fn can_go_forward(&self) -> bool {
        if let Some(current) = self.current_index {
            current < self.entries.len() - 1
        } else {
            false
        }
    }

    fn go_back(&mut self) -> Option<&HistoryEntry> {
        if let Some(current) = self.current_index {
            if current > 0 {
                self.current_index = Some(current - 1);
                return Some(&self.entries[current - 1]);
            }
        }
        None
    }

    fn go_forward(&mut self) -> Option<&HistoryEntry> {
        if let Some(current) = self.current_index {
            if current < self.entries.len() - 1 {
                self.current_index = Some(current + 1);
                return Some(&self.entries[current + 1]);
            }
        }
        None
    }
}

/// Tab state including navigation history
#[derive(Debug, Clone)]
struct TabState {
    tab: Tab,
    history: NavigationHistory,
}

/// TabInfo is the public representation of tab state
#[derive(Debug, Clone)]
pub struct TabInfo {
    pub id: TabId,
    pub window_id: WindowId,
    pub title: String,
    pub url: Option<Url>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// Whether this tab is in private/incognito mode
    pub is_private: bool,
}

impl From<&Tab> for TabInfo {
    fn from(tab: &Tab) -> Self {
        Self {
            id: tab.id,
            window_id: tab.window_id,
            title: tab.title.clone(),
            url: tab.url.clone(),
            loading: tab.loading,
            can_go_back: tab.can_go_back,
            can_go_forward: tab.can_go_forward,
            is_private: tab.is_private,
        }
    }
}

/// In-memory storage for private session data.
/// This data is never persisted and is cleared when the tab closes.
#[derive(Debug, Clone, Default)]
pub struct PrivateSessionData {
    /// In-memory cookies for the private session
    pub cookies: HashMap<String, String>,
    /// In-memory cache entries for the private session
    pub cache: HashMap<String, Vec<u8>>,
    /// In-memory form data for the private session
    pub form_data: HashMap<String, String>,
}

impl PrivateSessionData {
    /// Create a new empty private session data store
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all session data
    pub fn clear(&mut self) {
        self.cookies.clear();
        self.cache.clear();
        self.form_data.clear();
    }
}

/// TabManager manages all tabs and their lifecycles
pub struct TabManager {
    tabs: HashMap<TabId, TabState>,
    /// Private session data keyed by tab ID.
    /// Only populated for private/incognito tabs.
    private_sessions: HashMap<TabId, PrivateSessionData>,
}

impl TabManager {
    /// Create a new TabManager
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            private_sessions: HashMap::new(),
        }
    }

    /// Get the number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Create a new tab in a window
    pub async fn create_tab(
        &mut self,
        window_id: WindowId,
        url: Option<String>,
    ) -> Result<TabId, TabError> {
        let tab_id = TabId::new();
        let render_surface = RenderSurfaceId::new();

        let parsed_url = if let Some(url_str) = url {
            Some(
                Url::parse(&url_str)
                    .map_err(|e| TabError::CreationFailed(format!("Invalid URL: {}", e)))?,
            )
        } else {
            None
        };

        let mut history = NavigationHistory::new();

        // If we have a URL, add it to history
        if let Some(ref url) = parsed_url {
            history.push(url.clone(), String::new());
        }

        let tab = Tab {
            id: tab_id,
            window_id,
            title: String::new(),
            url: parsed_url,
            loading: false,
            can_go_back: history.can_go_back(),
            can_go_forward: history.can_go_forward(),
            favicon: None,
            process_id: None,
            render_surface,
            is_private: false,
        };

        self.tabs.insert(tab_id, TabState { tab, history });

        Ok(tab_id)
    }

    /// Create a new private/incognito tab in a window.
    ///
    /// Private tabs have the following characteristics:
    /// - No history is recorded
    /// - No cookies are persisted to disk
    /// - No cache is persisted to disk
    /// - All session data is stored in-memory only
    /// - All private data is cleared when the tab closes
    pub async fn create_private_tab(
        &mut self,
        window_id: WindowId,
        url: Option<String>,
    ) -> Result<TabId, TabError> {
        let tab_id = TabId::new();
        let render_surface = RenderSurfaceId::new();

        let parsed_url = if let Some(url_str) = url {
            Some(
                Url::parse(&url_str)
                    .map_err(|e| TabError::CreationFailed(format!("Invalid URL: {}", e)))?,
            )
        } else {
            None
        };

        let mut history = NavigationHistory::new();

        // If we have a URL, add it to local navigation history
        // (not persisted to global history)
        if let Some(ref url) = parsed_url {
            history.push(url.clone(), String::new());
        }

        let tab = Tab {
            id: tab_id,
            window_id,
            title: String::new(),
            url: parsed_url,
            loading: false,
            can_go_back: history.can_go_back(),
            can_go_forward: history.can_go_forward(),
            favicon: None,
            process_id: None,
            render_surface,
            is_private: true,
        };

        self.tabs.insert(tab_id, TabState { tab, history });

        // Initialize private session data for this tab
        self.private_sessions.insert(tab_id, PrivateSessionData::new());

        Ok(tab_id)
    }

    /// Close a tab
    ///
    /// For private tabs, this also clears all associated private session data.
    pub async fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        let state = self.tabs
            .remove(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        // If this was a private tab, clean up its session data
        if state.tab.is_private {
            self.private_sessions.remove(&tab_id);
        }

        Ok(())
    }

    /// Navigate a tab to a new URL
    pub async fn navigate(&mut self, tab_id: TabId, url: String) -> Result<(), TabError> {
        let state = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        let parsed_url = Url::parse(&url)
            .map_err(|e| TabError::NavigationFailed(format!("Invalid URL: {}", e)))?;

        // Add to history
        state.history.push(parsed_url.clone(), String::new());

        // Update tab
        state.tab.url = Some(parsed_url);
        state.tab.loading = false;
        state.tab.can_go_back = state.history.can_go_back();
        state.tab.can_go_forward = state.history.can_go_forward();

        Ok(())
    }

    /// Reload a tab
    pub async fn reload(&mut self, tab_id: TabId, _ignore_cache: bool) -> Result<(), TabError> {
        let state = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        // Reload simulation - in real implementation this would trigger page reload
        state.tab.loading = false;

        Ok(())
    }

    /// Stop loading a tab
    pub async fn stop(&mut self, tab_id: TabId) -> Result<(), TabError> {
        let state = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        state.tab.loading = false;

        Ok(())
    }

    /// Navigate back in history
    pub async fn go_back(&mut self, tab_id: TabId) -> Result<(), TabError> {
        let state = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        let entry = state
            .history
            .go_back()
            .ok_or_else(|| TabError::NavigationFailed("No back history".to_string()))?;

        state.tab.url = Some(entry.url.clone());
        state.tab.title = entry.title.clone();
        state.tab.can_go_back = state.history.can_go_back();
        state.tab.can_go_forward = state.history.can_go_forward();

        Ok(())
    }

    /// Navigate forward in history
    pub async fn go_forward(&mut self, tab_id: TabId) -> Result<(), TabError> {
        let state = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        let entry = state
            .history
            .go_forward()
            .ok_or_else(|| TabError::NavigationFailed("No forward history".to_string()))?;

        state.tab.url = Some(entry.url.clone());
        state.tab.title = entry.title.clone();
        state.tab.can_go_back = state.history.can_go_back();
        state.tab.can_go_forward = state.history.can_go_forward();

        Ok(())
    }

    /// Get tab information
    pub fn get_tab_info(&self, tab_id: TabId) -> Option<TabInfo> {
        self.tabs
            .get(&tab_id)
            .map(|state| TabInfo::from(&state.tab))
    }

    /// Check if a tab is in private/incognito mode
    pub fn is_private(&self, tab_id: TabId) -> bool {
        self.tabs
            .get(&tab_id)
            .map(|state| state.tab.is_private)
            .unwrap_or(false)
    }

    /// Get the number of private tabs currently open
    pub fn private_tab_count(&self) -> usize {
        self.tabs
            .values()
            .filter(|state| state.tab.is_private)
            .count()
    }

    /// Get mutable access to a private tab's session data.
    ///
    /// Returns None if the tab doesn't exist or is not private.
    pub fn get_private_session_mut(&mut self, tab_id: TabId) -> Option<&mut PrivateSessionData> {
        // Only return session data if the tab exists and is private
        if self.is_private(tab_id) {
            self.private_sessions.get_mut(&tab_id)
        } else {
            None
        }
    }

    /// Get read-only access to a private tab's session data.
    ///
    /// Returns None if the tab doesn't exist or is not private.
    pub fn get_private_session(&self, tab_id: TabId) -> Option<&PrivateSessionData> {
        // Only return session data if the tab exists and is private
        if self.is_private(tab_id) {
            self.private_sessions.get(&tab_id)
        } else {
            None
        }
    }

    /// Clear all private session data for all private tabs.
    ///
    /// This does not close the tabs, but clears their cookies, cache, and form data.
    pub fn clear_all_private_data(&mut self) {
        for session in self.private_sessions.values_mut() {
            session.clear();
        }
    }

    /// Get all private tab IDs
    pub fn get_private_tab_ids(&self) -> Vec<TabId> {
        self.tabs
            .values()
            .filter(|state| state.tab.is_private)
            .map(|state| state.tab.id)
            .collect()
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history_basic() {
        let mut history = NavigationHistory::new();

        assert_eq!(history.can_go_back(), false);
        assert_eq!(history.can_go_forward(), false);

        let url1 = Url::parse("https://example.com").unwrap();
        history.push(url1.clone(), "Example".to_string());

        assert_eq!(history.can_go_back(), false);
        assert_eq!(history.can_go_forward(), false);

        let url2 = Url::parse("https://example.org").unwrap();
        history.push(url2.clone(), "Example Org".to_string());

        assert_eq!(history.can_go_back(), true);
        assert_eq!(history.can_go_forward(), false);
    }

    #[test]
    fn test_navigation_history_back_forward() {
        let mut history = NavigationHistory::new();

        let url1 = Url::parse("https://example.com").unwrap();
        let url2 = Url::parse("https://example.org").unwrap();

        history.push(url1.clone(), "Example".to_string());
        history.push(url2.clone(), "Example Org".to_string());

        let entry = history.go_back().unwrap();
        assert_eq!(entry.url, url1);

        assert_eq!(history.can_go_back(), false);
        assert_eq!(history.can_go_forward(), true);

        let entry = history.go_forward().unwrap();
        assert_eq!(entry.url, url2);

        assert_eq!(history.can_go_back(), true);
        assert_eq!(history.can_go_forward(), false);
    }

    #[test]
    fn test_navigation_history_truncate_on_navigate() {
        let mut history = NavigationHistory::new();

        let url1 = Url::parse("https://example.com").unwrap();
        let url2 = Url::parse("https://example.org").unwrap();
        let url3 = Url::parse("https://example.net").unwrap();

        history.push(url1.clone(), "1".to_string());
        history.push(url2.clone(), "2".to_string());

        history.go_back();
        assert_eq!(history.can_go_forward(), true);

        // This should truncate the forward history
        history.push(url3.clone(), "3".to_string());

        assert_eq!(history.can_go_forward(), false);
        assert_eq!(history.can_go_back(), true);

        let entry = history.go_back().unwrap();
        assert_eq!(entry.url, url1);
    }

    #[tokio::test]
    async fn test_create_private_tab() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_private_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        assert!(manager.is_private(tab_id));
        assert_eq!(manager.private_tab_count(), 1);

        let info = manager.get_tab_info(tab_id).unwrap();
        assert!(info.is_private);
        assert_eq!(info.url.unwrap().as_str(), "https://example.com/");
    }

    #[tokio::test]
    async fn test_regular_tab_not_private() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        assert!(!manager.is_private(tab_id));
        assert_eq!(manager.private_tab_count(), 0);

        let info = manager.get_tab_info(tab_id).unwrap();
        assert!(!info.is_private);
    }

    #[tokio::test]
    async fn test_private_session_data() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_private_tab(window_id, None)
            .await
            .unwrap();

        // Private tab should have session data
        let session = manager.get_private_session_mut(tab_id).unwrap();
        session.cookies.insert("test_cookie".to_string(), "value".to_string());
        session.cache.insert("test_key".to_string(), vec![1, 2, 3]);

        // Verify data was stored
        let session = manager.get_private_session(tab_id).unwrap();
        assert_eq!(session.cookies.get("test_cookie"), Some(&"value".to_string()));
        assert_eq!(session.cache.get("test_key"), Some(&vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_regular_tab_no_private_session() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, None)
            .await
            .unwrap();

        // Regular tab should not have private session data
        assert!(manager.get_private_session(tab_id).is_none());
        assert!(manager.get_private_session_mut(tab_id).is_none());
    }

    #[tokio::test]
    async fn test_close_private_tab_clears_data() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_private_tab(window_id, None)
            .await
            .unwrap();

        // Add some data to the private session
        {
            let session = manager.get_private_session_mut(tab_id).unwrap();
            session.cookies.insert("cookie".to_string(), "data".to_string());
        }

        // Verify private session exists
        assert!(manager.private_sessions.contains_key(&tab_id));

        // Close the tab
        manager.close_tab(tab_id).await.unwrap();

        // Private session data should be cleaned up
        assert!(!manager.private_sessions.contains_key(&tab_id));
        assert_eq!(manager.private_tab_count(), 0);
    }

    #[tokio::test]
    async fn test_clear_all_private_data() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        // Create two private tabs with data
        let tab1 = manager.create_private_tab(window_id, None).await.unwrap();
        let tab2 = manager.create_private_tab(window_id, None).await.unwrap();

        {
            let session1 = manager.get_private_session_mut(tab1).unwrap();
            session1.cookies.insert("cookie1".to_string(), "value1".to_string());
        }
        {
            let session2 = manager.get_private_session_mut(tab2).unwrap();
            session2.cookies.insert("cookie2".to_string(), "value2".to_string());
        }

        // Clear all private data
        manager.clear_all_private_data();

        // Both sessions should be empty but tabs still exist
        assert!(manager.get_private_session(tab1).unwrap().cookies.is_empty());
        assert!(manager.get_private_session(tab2).unwrap().cookies.is_empty());
        assert_eq!(manager.private_tab_count(), 2);
    }

    #[tokio::test]
    async fn test_get_private_tab_ids() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        // Create mix of regular and private tabs
        let _regular1 = manager.create_tab(window_id, None).await.unwrap();
        let private1 = manager.create_private_tab(window_id, None).await.unwrap();
        let _regular2 = manager.create_tab(window_id, None).await.unwrap();
        let private2 = manager.create_private_tab(window_id, None).await.unwrap();

        let private_ids = manager.get_private_tab_ids();
        assert_eq!(private_ids.len(), 2);
        assert!(private_ids.contains(&private1));
        assert!(private_ids.contains(&private2));
    }

    #[tokio::test]
    async fn test_private_tab_navigation() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_private_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // Navigate to another URL
        manager.navigate(tab_id, "https://example.org".to_string()).await.unwrap();

        let info = manager.get_tab_info(tab_id).unwrap();
        assert_eq!(info.url.unwrap().as_str(), "https://example.org/");
        assert!(info.can_go_back);
        assert!(info.is_private);
    }

    #[test]
    fn test_private_session_data_clear() {
        let mut session = PrivateSessionData::new();
        session.cookies.insert("key".to_string(), "value".to_string());
        session.cache.insert("key".to_string(), vec![1, 2, 3]);
        session.form_data.insert("field".to_string(), "data".to_string());

        assert!(!session.cookies.is_empty());
        assert!(!session.cache.is_empty());
        assert!(!session.form_data.is_empty());

        session.clear();

        assert!(session.cookies.is_empty());
        assert!(session.cache.is_empty());
        assert!(session.form_data.is_empty());
    }

    #[tokio::test]
    async fn test_is_private_nonexistent_tab() {
        let manager = TabManager::new();
        let fake_tab_id = TabId::new();

        // Should return false for non-existent tab
        assert!(!manager.is_private(fake_tab_id));
    }
}
