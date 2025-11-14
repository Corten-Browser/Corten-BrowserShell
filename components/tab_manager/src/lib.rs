//! Tab Manager Component
//!
//! This component manages tab lifecycle, navigation, and history tracking
//! for the CortenBrowser Browser Shell.

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
        }
    }
}

/// TabManager manages all tabs and their lifecycles
pub struct TabManager {
    tabs: HashMap<TabId, TabState>,
}

impl TabManager {
    /// Create a new TabManager
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
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
        };

        self.tabs.insert(tab_id, TabState { tab, history });

        Ok(tab_id)
    }

    /// Close a tab
    pub async fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        self.tabs
            .remove(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;
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
}
