// @implements: REQ-UI-002
//! Tab Bar Widget
//!
//! Tab strip with close buttons and tab management.

use std::collections::HashMap;

/// Tab ID type
pub type TabId = u64;

/// Information about a tab displayed in the tab bar
#[derive(Debug, Clone, PartialEq)]
pub struct TabInfo {
    pub title: String,
    pub url: Option<String>,
    pub favicon: Option<Vec<u8>>,
    pub loading: bool,
}

/// Error types for tab operations
#[derive(Debug, Clone, PartialEq)]
pub enum TabError {
    TabNotFound,
    InvalidTabId,
}

impl std::fmt::Display for TabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabError::TabNotFound => write!(f, "Tab not found"),
            TabError::InvalidTabId => write!(f, "Invalid tab ID"),
        }
    }
}

impl std::error::Error for TabError {}

/// Tab Bar widget state
#[derive(Debug, Clone)]
pub struct TabBar {
    tabs: HashMap<TabId, TabInfo>,
    tab_order: Vec<TabId>,
    active_tab: Option<TabId>,
    next_tab_id: TabId,
}

impl TabBar {
    /// Create a new Tab Bar widget
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            tab_order: Vec::new(),
            active_tab: None,
            next_tab_id: 1,
        }
    }

    /// Get the number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get the currently active tab ID
    pub fn active_tab(&self) -> Option<TabId> {
        self.active_tab
    }

    /// Add a new tab
    pub fn add_tab(&mut self, info: TabInfo) -> TabId {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        self.tabs.insert(tab_id, info);
        self.tab_order.push(tab_id);
        self.active_tab = Some(tab_id);

        tab_id
    }

    /// Close a tab
    pub fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        if !self.tabs.contains_key(&tab_id) {
            return Err(TabError::TabNotFound);
        }

        self.tabs.remove(&tab_id);
        self.tab_order.retain(|&id| id != tab_id);

        // If the closed tab was active, activate another tab
        if self.active_tab == Some(tab_id) {
            self.active_tab = self.tab_order.last().copied();
        }

        Ok(())
    }

    /// Set the active tab
    pub fn set_active_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        if !self.tabs.contains_key(&tab_id) {
            return Err(TabError::TabNotFound);
        }

        self.active_tab = Some(tab_id);
        Ok(())
    }

    /// Get tab information
    pub fn get_tab_info(&self, tab_id: TabId) -> Option<&TabInfo> {
        self.tabs.get(&tab_id)
    }

    /// Update tab information
    pub fn update_tab_info(&mut self, tab_id: TabId, info: TabInfo) -> Result<(), TabError> {
        if !self.tabs.contains_key(&tab_id) {
            return Err(TabError::TabNotFound);
        }

        self.tabs.insert(tab_id, info);
        Ok(())
    }

    /// Get all tabs in order
    pub fn get_all_tabs(&self) -> Vec<(TabId, &TabInfo)> {
        self.tab_order
            .iter()
            .filter_map(|&id| self.tabs.get(&id).map(|info| (id, info)))
            .collect()
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_bar_new_creates_empty() {
        let bar = TabBar::new();
        assert_eq!(bar.tab_count(), 0);
        assert!(bar.active_tab().is_none());
    }

    #[test]
    fn tab_bar_add_tab_increments_count() {
        let mut bar = TabBar::new();
        let info = TabInfo {
            title: "Test".to_string(),
            url: None,
            favicon: None,
            loading: false,
        };

        bar.add_tab(info);
        assert_eq!(bar.tab_count(), 1);
    }

    #[test]
    fn tab_bar_add_tab_sets_active() {
        let mut bar = TabBar::new();
        let info = TabInfo {
            title: "Test".to_string(),
            url: None,
            favicon: None,
            loading: false,
        };

        let tab_id = bar.add_tab(info);
        assert_eq!(bar.active_tab(), Some(tab_id));
    }

    #[test]
    fn tab_bar_close_tab_decrements_count() {
        let mut bar = TabBar::new();
        let info = TabInfo {
            title: "Test".to_string(),
            url: None,
            favicon: None,
            loading: false,
        };

        let tab_id = bar.add_tab(info);
        bar.close_tab(tab_id).unwrap();
        assert_eq!(bar.tab_count(), 0);
    }

    #[test]
    fn tab_bar_close_last_tab_clears_active() {
        let mut bar = TabBar::new();
        let info = TabInfo {
            title: "Test".to_string(),
            url: None,
            favicon: None,
            loading: false,
        };

        let tab_id = bar.add_tab(info);
        bar.close_tab(tab_id).unwrap();
        assert!(bar.active_tab().is_none());
    }

    #[test]
    fn tab_info_equality() {
        let info1 = TabInfo {
            title: "Test".to_string(),
            url: Some("https://example.com".to_string()),
            favicon: None,
            loading: false,
        };

        let info2 = TabInfo {
            title: "Test".to_string(),
            url: Some("https://example.com".to_string()),
            favicon: None,
            loading: false,
        };

        assert_eq!(info1, info2);
    }
}
