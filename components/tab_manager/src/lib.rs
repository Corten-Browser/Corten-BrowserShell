//! Tab Manager Component
//!
//! This component manages tab lifecycle, navigation, and history tracking
//! for the CortenBrowser Browser Shell.
//!
//! Supports private/incognito browsing mode where no data is persisted.
//!
//! ## Lazy Tab Loading
//!
//! Tabs support lazy loading to reduce memory usage. Tabs start in an
//! `Unloaded` state and only load content when activated. Inactive tabs
//! can be suspended to free memory while preserving URL and title.
//!
//! ## Session Management (Crash Recovery)
//!
//! The [`session`] module provides crash recovery functionality:
//!
//! - **Session persistence**: Save tab state periodically to disk
//! - **Crash detection**: Uses a dirty flag (lock file) to detect abnormal shutdown
//! - **Session restore**: Restore previous session on startup
//! - **Multiple windows**: Handles sessions with multiple windows and tabs
//!
//! ```rust,ignore
//! use tab_manager::session::{SessionManager, SessionConfig};
//!
//! let config = SessionConfig::with_session_dir("/path/to/session");
//! let mut manager = SessionManager::new(config);
//!
//! // Check if previous session crashed
//! if manager.was_crash().await {
//!     // Offer to restore session
//!     let session = manager.restore_session().await?;
//! }
//!
//! // Mark session as active (for crash detection)
//! manager.mark_session_active().await?;
//!
//! // Save session periodically
//! manager.save_session().await?;
//!
//! // On clean shutdown
//! manager.mark_session_closed().await?;
//! ```

pub mod session;

use shared_types::{ProcessId, RenderSurfaceId, TabError, TabId, WindowId};
use std::collections::HashMap;
use std::time::Instant;
use url::Url;

/// Tab content loading state for lazy loading support.
///
/// Tabs start in `Unloaded` state and only load content when activated.
/// This reduces memory usage when many tabs are open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabLoadState {
    /// URL known but content not loaded (initial state for new tabs)
    Unloaded,
    /// Currently loading content
    Loading,
    /// Fully loaded and rendered
    Loaded,
    /// Was loaded but suspended to save memory
    Suspended,
}

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
    /// Current loading state for lazy loading support.
    /// Tabs start as Unloaded and only load content when activated.
    pub load_state: TabLoadState,
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
    /// Timestamp of last access (for auto-suspend decisions)
    last_accessed: Instant,
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
    /// Current loading state for lazy loading support
    pub load_state: TabLoadState,
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
            load_state: tab.load_state,
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

/// Configuration for lazy tab loading behavior
#[derive(Debug, Clone)]
pub struct LazyLoadConfig {
    /// Number of loaded tabs before auto-suspending inactive ones.
    /// Set to 0 to disable auto-suspend.
    pub auto_suspend_threshold: usize,
    /// Whether to load tabs immediately when created (false = lazy loading)
    pub immediate_load: bool,
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self {
            auto_suspend_threshold: 10,  // Suspend when more than 10 tabs loaded
            immediate_load: false,       // Default to lazy loading
        }
    }
}

/// TabManager manages all tabs and their lifecycles
pub struct TabManager {
    tabs: HashMap<TabId, TabState>,
    /// Private session data keyed by tab ID.
    /// Only populated for private/incognito tabs.
    private_sessions: HashMap<TabId, PrivateSessionData>,
    /// Configuration for lazy tab loading
    lazy_load_config: LazyLoadConfig,
}

impl TabManager {
    /// Create a new TabManager with default lazy loading configuration
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            private_sessions: HashMap::new(),
            lazy_load_config: LazyLoadConfig::default(),
        }
    }

    /// Create a new TabManager with custom lazy loading configuration
    pub fn with_config(config: LazyLoadConfig) -> Self {
        Self {
            tabs: HashMap::new(),
            private_sessions: HashMap::new(),
            lazy_load_config: config,
        }
    }

    /// Get the number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Create a new tab in a window.
    ///
    /// By default, tabs are created in `Unloaded` state to support lazy loading.
    /// Call `load_tab()` to trigger content loading when the tab is activated.
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

        // Determine initial load state based on configuration
        let initial_load_state = if self.lazy_load_config.immediate_load {
            TabLoadState::Loaded
        } else {
            TabLoadState::Unloaded
        };

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
            load_state: initial_load_state,
        };

        self.tabs.insert(
            tab_id,
            TabState {
                tab,
                history,
                last_accessed: Instant::now(),
            },
        );

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
    ///
    /// Like regular tabs, private tabs start in `Unloaded` state by default.
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

        // Determine initial load state based on configuration
        let initial_load_state = if self.lazy_load_config.immediate_load {
            TabLoadState::Loaded
        } else {
            TabLoadState::Unloaded
        };

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
            load_state: initial_load_state,
        };

        self.tabs.insert(
            tab_id,
            TabState {
                tab,
                history,
                last_accessed: Instant::now(),
            },
        );

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

    // ==================== Lazy Loading API ====================

    /// Get the current load state of a tab.
    ///
    /// Returns `None` if the tab does not exist.
    pub fn get_load_state(&self, tab_id: TabId) -> Option<TabLoadState> {
        self.tabs.get(&tab_id).map(|state| state.tab.load_state)
    }

    /// Load a tab's content (transition from Unloaded/Suspended to Loading then Loaded).
    ///
    /// This method triggers content loading for a tab. It should be called when
    /// a tab is activated/selected by the user.
    ///
    /// # State Transitions
    /// - `Unloaded` -> `Loading` -> `Loaded`
    /// - `Suspended` -> `Loading` -> `Loaded`
    /// - `Loading` -> No change (already loading)
    /// - `Loaded` -> No change (already loaded)
    ///
    /// # Auto-suspend behavior
    /// If auto-suspend is enabled and the number of loaded tabs exceeds the threshold,
    /// the least recently accessed tabs will be suspended.
    pub async fn load_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        let state = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        // Update last accessed time
        state.last_accessed = Instant::now();

        match state.tab.load_state {
            TabLoadState::Loaded => {
                // Already loaded, nothing to do
                return Ok(());
            }
            TabLoadState::Loading => {
                // Already loading, nothing to do
                return Ok(());
            }
            TabLoadState::Unloaded | TabLoadState::Suspended => {
                // Transition to Loading
                state.tab.load_state = TabLoadState::Loading;
                state.tab.loading = true;
            }
        }

        // Simulate loading completion (in real implementation, this would be async)
        // For now, we immediately transition to Loaded
        let state = self.tabs.get_mut(&tab_id).unwrap();
        state.tab.load_state = TabLoadState::Loaded;
        state.tab.loading = false;

        // Check if we need to auto-suspend other tabs
        self.auto_suspend_if_needed(tab_id);

        Ok(())
    }

    /// Suspend a tab to free memory while preserving URL and title.
    ///
    /// Suspended tabs keep their URL and title but release their content
    /// from memory. They can be reloaded later with `load_tab()`.
    ///
    /// # State Transitions
    /// - `Loaded` -> `Suspended`
    /// - `Loading` -> `Suspended` (cancels loading)
    /// - `Unloaded` -> No change
    /// - `Suspended` -> No change
    ///
    /// # Errors
    /// Returns `TabError::NotFound` if the tab does not exist.
    pub async fn suspend_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        let state = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        match state.tab.load_state {
            TabLoadState::Loaded | TabLoadState::Loading => {
                state.tab.load_state = TabLoadState::Suspended;
                state.tab.loading = false;
                // In a real implementation, we would release content memory here
                // but preserve URL, title, favicon, and history
            }
            TabLoadState::Unloaded | TabLoadState::Suspended => {
                // Already unloaded or suspended, nothing to do
            }
        }

        Ok(())
    }

    /// Set the auto-suspend threshold.
    ///
    /// When the number of loaded tabs exceeds this threshold, the least
    /// recently accessed tabs will be automatically suspended.
    ///
    /// Set to 0 to disable auto-suspend.
    pub fn set_auto_suspend_threshold(&mut self, count: usize) {
        self.lazy_load_config.auto_suspend_threshold = count;
    }

    /// Get the current auto-suspend threshold.
    pub fn get_auto_suspend_threshold(&self) -> usize {
        self.lazy_load_config.auto_suspend_threshold
    }

    /// Get the number of currently loaded tabs.
    pub fn loaded_tab_count(&self) -> usize {
        self.tabs
            .values()
            .filter(|state| state.tab.load_state == TabLoadState::Loaded)
            .count()
    }

    /// Get the number of currently suspended tabs.
    pub fn suspended_tab_count(&self) -> usize {
        self.tabs
            .values()
            .filter(|state| state.tab.load_state == TabLoadState::Suspended)
            .count()
    }

    /// Get the number of unloaded tabs.
    pub fn unloaded_tab_count(&self) -> usize {
        self.tabs
            .values()
            .filter(|state| state.tab.load_state == TabLoadState::Unloaded)
            .count()
    }

    /// Get all tab IDs by their load state.
    pub fn get_tabs_by_load_state(&self, load_state: TabLoadState) -> Vec<TabId> {
        self.tabs
            .values()
            .filter(|state| state.tab.load_state == load_state)
            .map(|state| state.tab.id)
            .collect()
    }

    /// Automatically suspend least recently accessed tabs if threshold is exceeded.
    ///
    /// This is called internally after loading a tab.
    fn auto_suspend_if_needed(&mut self, exclude_tab_id: TabId) {
        let threshold = self.lazy_load_config.auto_suspend_threshold;

        // Auto-suspend disabled if threshold is 0
        if threshold == 0 {
            return;
        }

        let loaded_count = self.loaded_tab_count();

        if loaded_count <= threshold {
            return;
        }

        // Find tabs to suspend (least recently accessed, excluding the just-loaded tab)
        let mut loaded_tabs: Vec<_> = self
            .tabs
            .iter()
            .filter(|(id, state)| {
                **id != exclude_tab_id && state.tab.load_state == TabLoadState::Loaded
            })
            .map(|(id, state)| (*id, state.last_accessed))
            .collect();

        // Sort by last_accessed (oldest first)
        loaded_tabs.sort_by(|a, b| a.1.cmp(&b.1));

        // Suspend oldest tabs until we're at or below threshold
        let tabs_to_suspend = loaded_count - threshold;
        for (tab_id, _) in loaded_tabs.into_iter().take(tabs_to_suspend) {
            if let Some(state) = self.tabs.get_mut(&tab_id) {
                state.tab.load_state = TabLoadState::Suspended;
                state.tab.loading = false;
            }
        }
    }

    /// Get the lazy load configuration.
    pub fn get_lazy_load_config(&self) -> &LazyLoadConfig {
        &self.lazy_load_config
    }

    /// Set immediate load mode (disable lazy loading for new tabs).
    pub fn set_immediate_load(&mut self, immediate: bool) {
        self.lazy_load_config.immediate_load = immediate;
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

    // ==================== Lazy Loading Tests ====================

    #[tokio::test]
    async fn test_new_tab_starts_unloaded() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // New tabs should start in Unloaded state by default
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Unloaded));

        let info = manager.get_tab_info(tab_id).unwrap();
        assert_eq!(info.load_state, TabLoadState::Unloaded);
        // URL should be preserved even when unloaded
        assert_eq!(info.url.unwrap().as_str(), "https://example.com/");
    }

    #[tokio::test]
    async fn test_load_tab_transitions_to_loaded() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // Initially unloaded
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Unloaded));

        // Load the tab
        manager.load_tab(tab_id).await.unwrap();

        // Now should be loaded
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Loaded));
    }

    #[tokio::test]
    async fn test_suspend_tab_transitions_to_suspended() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // Load then suspend
        manager.load_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Loaded));

        manager.suspend_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Suspended));

        // URL should still be preserved
        let info = manager.get_tab_info(tab_id).unwrap();
        assert_eq!(info.url.unwrap().as_str(), "https://example.com/");
    }

    #[tokio::test]
    async fn test_reload_suspended_tab() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // Load -> Suspend -> Load again
        manager.load_tab(tab_id).await.unwrap();
        manager.suspend_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Suspended));

        manager.load_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Loaded));
    }

    #[tokio::test]
    async fn test_suspend_unloaded_tab_no_change() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // Suspending an unloaded tab should have no effect
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Unloaded));
        manager.suspend_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Unloaded));
    }

    #[tokio::test]
    async fn test_load_already_loaded_tab_no_change() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        manager.load_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Loaded));

        // Loading again should be a no-op
        manager.load_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Loaded));
    }

    #[tokio::test]
    async fn test_load_state_counts() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        // Create 3 tabs
        let tab1 = manager.create_tab(window_id, None).await.unwrap();
        let tab2 = manager.create_tab(window_id, None).await.unwrap();
        let tab3 = manager.create_tab(window_id, None).await.unwrap();

        // All start unloaded
        assert_eq!(manager.unloaded_tab_count(), 3);
        assert_eq!(manager.loaded_tab_count(), 0);
        assert_eq!(manager.suspended_tab_count(), 0);

        // Load tab1 and tab2
        manager.load_tab(tab1).await.unwrap();
        manager.load_tab(tab2).await.unwrap();

        assert_eq!(manager.unloaded_tab_count(), 1);
        assert_eq!(manager.loaded_tab_count(), 2);
        assert_eq!(manager.suspended_tab_count(), 0);

        // Suspend tab1
        manager.suspend_tab(tab1).await.unwrap();

        assert_eq!(manager.unloaded_tab_count(), 1);
        assert_eq!(manager.loaded_tab_count(), 1);
        assert_eq!(manager.suspended_tab_count(), 1);

        // Load tab3
        manager.load_tab(tab3).await.unwrap();

        assert_eq!(manager.unloaded_tab_count(), 0);
        assert_eq!(manager.loaded_tab_count(), 2);
        assert_eq!(manager.suspended_tab_count(), 1);
    }

    #[tokio::test]
    async fn test_get_tabs_by_load_state() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab1 = manager.create_tab(window_id, None).await.unwrap();
        let tab2 = manager.create_tab(window_id, None).await.unwrap();
        let tab3 = manager.create_tab(window_id, None).await.unwrap();

        manager.load_tab(tab1).await.unwrap();
        manager.load_tab(tab2).await.unwrap();
        manager.suspend_tab(tab1).await.unwrap();

        let unloaded = manager.get_tabs_by_load_state(TabLoadState::Unloaded);
        let loaded = manager.get_tabs_by_load_state(TabLoadState::Loaded);
        let suspended = manager.get_tabs_by_load_state(TabLoadState::Suspended);

        assert_eq!(unloaded.len(), 1);
        assert!(unloaded.contains(&tab3));

        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains(&tab2));

        assert_eq!(suspended.len(), 1);
        assert!(suspended.contains(&tab1));
    }

    #[tokio::test]
    async fn test_auto_suspend_threshold() {
        // Create manager with threshold of 2
        let config = LazyLoadConfig {
            auto_suspend_threshold: 2,
            immediate_load: false,
        };
        let mut manager = TabManager::with_config(config);
        let window_id = WindowId::new();

        // Create and load 3 tabs
        let tab1 = manager.create_tab(window_id, None).await.unwrap();
        let tab2 = manager.create_tab(window_id, None).await.unwrap();
        let tab3 = manager.create_tab(window_id, None).await.unwrap();

        // Load tabs with small delays to establish access order
        manager.load_tab(tab1).await.unwrap();
        manager.load_tab(tab2).await.unwrap();

        // At threshold, no auto-suspend yet
        assert_eq!(manager.loaded_tab_count(), 2);

        // Loading tab3 should trigger auto-suspend of oldest (tab1)
        manager.load_tab(tab3).await.unwrap();

        assert_eq!(manager.loaded_tab_count(), 2);
        assert_eq!(manager.suspended_tab_count(), 1);

        // tab1 should be suspended (oldest)
        assert_eq!(manager.get_load_state(tab1), Some(TabLoadState::Suspended));
        // tab2 and tab3 should still be loaded
        assert_eq!(manager.get_load_state(tab2), Some(TabLoadState::Loaded));
        assert_eq!(manager.get_load_state(tab3), Some(TabLoadState::Loaded));
    }

    #[tokio::test]
    async fn test_auto_suspend_disabled_with_zero_threshold() {
        let config = LazyLoadConfig {
            auto_suspend_threshold: 0,
            immediate_load: false,
        };
        let mut manager = TabManager::with_config(config);
        let window_id = WindowId::new();

        // Create and load many tabs
        let tab1 = manager.create_tab(window_id, None).await.unwrap();
        let tab2 = manager.create_tab(window_id, None).await.unwrap();
        let tab3 = manager.create_tab(window_id, None).await.unwrap();

        manager.load_tab(tab1).await.unwrap();
        manager.load_tab(tab2).await.unwrap();
        manager.load_tab(tab3).await.unwrap();

        // All should remain loaded (no auto-suspend when threshold is 0)
        assert_eq!(manager.loaded_tab_count(), 3);
        assert_eq!(manager.suspended_tab_count(), 0);
    }

    #[tokio::test]
    async fn test_set_auto_suspend_threshold() {
        let mut manager = TabManager::new();

        // Default threshold is 10
        assert_eq!(manager.get_auto_suspend_threshold(), 10);

        manager.set_auto_suspend_threshold(5);
        assert_eq!(manager.get_auto_suspend_threshold(), 5);

        manager.set_auto_suspend_threshold(0);
        assert_eq!(manager.get_auto_suspend_threshold(), 0);
    }

    #[tokio::test]
    async fn test_immediate_load_mode() {
        let config = LazyLoadConfig {
            auto_suspend_threshold: 10,
            immediate_load: true,
        };
        let mut manager = TabManager::with_config(config);
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // With immediate_load=true, tabs should start Loaded
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Loaded));
    }

    #[tokio::test]
    async fn test_set_immediate_load() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        // Default: lazy loading enabled
        let tab1 = manager.create_tab(window_id, None).await.unwrap();
        assert_eq!(manager.get_load_state(tab1), Some(TabLoadState::Unloaded));

        // Enable immediate loading
        manager.set_immediate_load(true);

        let tab2 = manager.create_tab(window_id, None).await.unwrap();
        assert_eq!(manager.get_load_state(tab2), Some(TabLoadState::Loaded));

        // Disable immediate loading
        manager.set_immediate_load(false);

        let tab3 = manager.create_tab(window_id, None).await.unwrap();
        assert_eq!(manager.get_load_state(tab3), Some(TabLoadState::Unloaded));
    }

    #[tokio::test]
    async fn test_load_nonexistent_tab_returns_error() {
        let mut manager = TabManager::new();
        let fake_tab_id = TabId::new();

        let result = manager.load_tab(fake_tab_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_suspend_nonexistent_tab_returns_error() {
        let mut manager = TabManager::new();
        let fake_tab_id = TabId::new();

        let result = manager.suspend_tab(fake_tab_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_load_state_nonexistent_tab_returns_none() {
        let manager = TabManager::new();
        let fake_tab_id = TabId::new();

        assert_eq!(manager.get_load_state(fake_tab_id), None);
    }

    #[tokio::test]
    async fn test_tab_info_includes_load_state() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // Check TabInfo includes load_state
        let info = manager.get_tab_info(tab_id).unwrap();
        assert_eq!(info.load_state, TabLoadState::Unloaded);

        manager.load_tab(tab_id).await.unwrap();
        let info = manager.get_tab_info(tab_id).unwrap();
        assert_eq!(info.load_state, TabLoadState::Loaded);

        manager.suspend_tab(tab_id).await.unwrap();
        let info = manager.get_tab_info(tab_id).unwrap();
        assert_eq!(info.load_state, TabLoadState::Suspended);
    }

    #[tokio::test]
    async fn test_private_tab_lazy_loading() {
        let mut manager = TabManager::new();
        let window_id = WindowId::new();

        let tab_id = manager
            .create_private_tab(window_id, Some("https://example.com".to_string()))
            .await
            .unwrap();

        // Private tabs should also start unloaded
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Unloaded));
        assert!(manager.is_private(tab_id));

        // Load and verify
        manager.load_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Loaded));

        // Suspend and verify
        manager.suspend_tab(tab_id).await.unwrap();
        assert_eq!(manager.get_load_state(tab_id), Some(TabLoadState::Suspended));

        // Should still be private
        assert!(manager.is_private(tab_id));
    }

    #[test]
    fn test_tab_load_state_enum() {
        // Verify enum variants exist and are distinct
        let unloaded = TabLoadState::Unloaded;
        let loading = TabLoadState::Loading;
        let loaded = TabLoadState::Loaded;
        let suspended = TabLoadState::Suspended;

        assert_ne!(unloaded, loading);
        assert_ne!(unloaded, loaded);
        assert_ne!(unloaded, suspended);
        assert_ne!(loading, loaded);
        assert_ne!(loading, suspended);
        assert_ne!(loaded, suspended);

        // Verify Debug trait
        assert_eq!(format!("{:?}", unloaded), "Unloaded");
        assert_eq!(format!("{:?}", loading), "Loading");
        assert_eq!(format!("{:?}", loaded), "Loaded");
        assert_eq!(format!("{:?}", suspended), "Suspended");
    }

    #[test]
    fn test_lazy_load_config_default() {
        let config = LazyLoadConfig::default();
        assert_eq!(config.auto_suspend_threshold, 10);
        assert!(!config.immediate_load);
    }
}
