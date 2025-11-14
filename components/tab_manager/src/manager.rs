// @implements: REQ-002, REQ-003
//! Core tab manager implementation
//!
//! Implements the TabManager trait with high-performance tab management.

use async_trait::async_trait;
use hashbrown::HashMap;
use shared_types::{
    Tab, TabError, TabId, TabManager, Url, WindowId, RenderSurfaceId,
};
use crate::navigation::NavigationHistory;
use crate::process::MockProcessManager;

/// High-performance tab manager implementation
///
/// Supports 500+ tabs with fast lookups and tab switching < 10ms.
#[derive(Debug)]
pub struct TabManagerImpl {
    /// All tabs indexed by TabId for O(1) lookup
    tabs: HashMap<TabId, Tab>,

    /// Navigation history per tab
    navigation: HashMap<TabId, NavigationHistory>,

    /// Active tab per window for fast activation
    active_tabs: HashMap<WindowId, TabId>,

    /// Process manager (mocked)
    process_manager: MockProcessManager,
}

impl TabManagerImpl {
    /// Create a new tab manager
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            navigation: HashMap::new(),
            active_tabs: HashMap::new(),
            process_manager: MockProcessManager::new(),
        }
    }

    /// Get the total number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if a tab is the active tab in its window
    pub fn is_active_tab(&self, tab_id: TabId) -> bool {
        self.tabs.get(&tab_id)
            .and_then(|tab| self.active_tabs.get(&tab.window_id))
            .map(|active_id| *active_id == tab_id)
            .unwrap_or(false)
    }

    /// Update tab's navigation state
    fn update_tab_navigation(&mut self, tab_id: TabId) {
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            if let Some(nav) = self.navigation.get(&tab_id) {
                tab.can_go_back = nav.can_go_back();
                tab.can_go_forward = nav.can_go_forward();
                tab.url = nav.current_url().cloned();
            }
        }
    }
}

impl Default for TabManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TabManager for TabManagerImpl {
    async fn create_tab(
        &mut self,
        window_id: WindowId,
        url: Option<Url>,
    ) -> Result<TabId, TabError> {
        let tab_id = TabId::new();

        // Allocate process
        let process_id = self.process_manager.allocate_process(tab_id);

        // Create navigation history
        let mut nav_history = NavigationHistory::new();
        if let Some(ref url) = url {
            nav_history.navigate(url.clone());
        }

        // Create tab
        let tab = Tab {
            id: tab_id,
            window_id,
            title: String::from("New Tab"),
            url: nav_history.current_url().cloned(),
            loading: false,
            can_go_back: nav_history.can_go_back(),
            can_go_forward: nav_history.can_go_forward(),
            favicon: None,
            process_id: Some(process_id),
            render_surface: RenderSurfaceId(tab_id.0 as u64),
        };

        // Insert tab and navigation history
        self.tabs.insert(tab_id, tab);
        self.navigation.insert(tab_id, nav_history);

        // If this is the first tab in the window, make it active
        if !self.active_tabs.contains_key(&window_id) {
            self.active_tabs.insert(window_id, tab_id);
        }

        Ok(tab_id)
    }

    async fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        // Check if tab exists
        let tab = self.tabs.get(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        let window_id = tab.window_id;

        // Release process
        self.process_manager.release_process(tab_id);

        // Remove tab and navigation
        self.tabs.remove(&tab_id);
        self.navigation.remove(&tab_id);

        // If this was the active tab, clear it
        if self.active_tabs.get(&window_id) == Some(&tab_id) {
            self.active_tabs.remove(&window_id);

            // Try to activate another tab in the same window
            if let Some(new_active) = self.tabs.values()
                .find(|t| t.window_id == window_id)
                .map(|t| t.id)
            {
                self.active_tabs.insert(window_id, new_active);
            }
        }

        Ok(())
    }

    async fn navigate(&mut self, tab_id: TabId, url: Url) -> Result<(), TabError> {
        // Check tab exists
        if !self.tabs.contains_key(&tab_id) {
            return Err(TabError::NotFound(tab_id));
        }

        // Update navigation history
        let nav = self.navigation.get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        nav.navigate(url.clone());

        // Update tab
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.loading = true;
            tab.url = Some(url);
        }

        self.update_tab_navigation(tab_id);

        Ok(())
    }

    async fn reload(&mut self, tab_id: TabId, _ignore_cache: bool) -> Result<(), TabError> {
        let tab = self.tabs.get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        // Mark as loading (in real browser would reload from network)
        tab.loading = true;

        Ok(())
    }

    async fn stop(&mut self, tab_id: TabId) -> Result<(), TabError> {
        let tab = self.tabs.get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        tab.loading = false;

        Ok(())
    }

    async fn go_back(&mut self, tab_id: TabId) -> Result<(), TabError> {
        // Update navigation history
        let nav = self.navigation.get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        let prev_url = nav.go_back()
            .ok_or_else(|| TabError::NavigationFailed(
                "No back history available".to_string()
            ))?;

        // Update tab
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.loading = true;
            tab.url = Some(prev_url);
        }

        self.update_tab_navigation(tab_id);

        Ok(())
    }

    async fn go_forward(&mut self, tab_id: TabId) -> Result<(), TabError> {
        // Update navigation history
        let nav = self.navigation.get_mut(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        let next_url = nav.go_forward()
            .ok_or_else(|| TabError::NavigationFailed(
                "No forward history available".to_string()
            ))?;

        // Update tab
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.loading = true;
            tab.url = Some(next_url);
        }

        self.update_tab_navigation(tab_id);

        Ok(())
    }

    fn get_tab(&self, tab_id: TabId) -> Option<&Tab> {
        self.tabs.get(&tab_id)
    }

    fn get_tabs(&self, window_id: WindowId) -> Vec<&Tab> {
        self.tabs.values()
            .filter(|tab| tab.window_id == window_id)
            .collect()
    }

    async fn activate_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        // Check tab exists
        let tab = self.tabs.get(&tab_id)
            .ok_or(TabError::NotFound(tab_id))?;

        let window_id = tab.window_id;

        // Update active tab for window
        self.active_tabs.insert(window_id, tab_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_manager_is_empty() {
        let manager = TabManagerImpl::new();
        assert_eq!(manager.tab_count(), 0);
    }

    #[tokio::test]
    async fn test_create_tab_increments_count() {
        let mut manager = TabManagerImpl::new();
        let window_id = WindowId::new();

        manager.create_tab(window_id, None).await.unwrap();
        assert_eq!(manager.tab_count(), 1);
    }

    #[tokio::test]
    async fn test_close_tab_decrements_count() {
        let mut manager = TabManagerImpl::new();
        let window_id = WindowId::new();

        let tab_id = manager.create_tab(window_id, None).await.unwrap();
        assert_eq!(manager.tab_count(), 1);

        manager.close_tab(tab_id).await.unwrap();
        assert_eq!(manager.tab_count(), 0);
    }
}
