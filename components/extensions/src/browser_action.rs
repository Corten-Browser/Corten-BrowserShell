//! Browser Action API
//!
//! Provides toolbar button functionality for extensions (Manifest V3 action API).

use crate::types::ExtensionId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Browser action (toolbar button) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAction {
    /// Default icon paths (size -> path)
    pub default_icon: HashMap<u32, String>,
    /// Default tooltip title
    pub default_title: String,
    /// Default popup HTML page
    pub default_popup: Option<String>,
}

impl BrowserAction {
    /// Create a new browser action
    pub fn new(title: String) -> Self {
        Self {
            default_icon: HashMap::new(),
            default_title: title,
            default_popup: None,
        }
    }

    /// Set the default popup
    pub fn with_popup(mut self, popup_path: String) -> Self {
        self.default_popup = Some(popup_path);
        self
    }

    /// Add an icon size
    pub fn with_icon(mut self, size: u32, path: String) -> Self {
        self.default_icon.insert(size, path);
        self
    }
}

impl Default for BrowserAction {
    fn default() -> Self {
        Self::new(String::new())
    }
}

/// Popup configuration for browser action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupConfig {
    /// HTML file path for the popup
    pub html_path: String,
    /// Width of the popup in pixels
    pub width: Option<u32>,
    /// Height of the popup in pixels
    pub height: Option<u32>,
}

impl PopupConfig {
    /// Create a new popup config
    pub fn new(html_path: String) -> Self {
        Self {
            html_path,
            width: None,
            height: None,
        }
    }

    /// Set dimensions
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// Runtime state of a browser action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserActionState {
    /// The base configuration
    pub config: BrowserAction,
    /// Badge text (shown on the icon)
    pub badge_text: String,
    /// Badge background color (CSS color)
    pub badge_background_color: String,
    /// Whether the action is enabled
    pub enabled: bool,
    /// Tab-specific titles (tab_id -> title)
    pub tab_titles: HashMap<u64, String>,
    /// Tab-specific icons (tab_id -> icon paths)
    pub tab_icons: HashMap<u64, HashMap<u32, String>>,
    /// Tab-specific popups (tab_id -> popup path)
    pub tab_popups: HashMap<u64, String>,
    /// Tab-specific badge text (tab_id -> text)
    pub tab_badge_text: HashMap<u64, String>,
}

impl BrowserActionState {
    /// Create state from configuration
    pub fn from_config(config: BrowserAction) -> Self {
        Self {
            config,
            badge_text: String::new(),
            badge_background_color: "#4688F1".to_string(),
            enabled: true,
            tab_titles: HashMap::new(),
            tab_icons: HashMap::new(),
            tab_popups: HashMap::new(),
            tab_badge_text: HashMap::new(),
        }
    }

    /// Get title for a specific tab, falling back to default
    pub fn get_title(&self, tab_id: Option<u64>) -> &str {
        if let Some(tid) = tab_id {
            if let Some(title) = self.tab_titles.get(&tid) {
                return title;
            }
        }
        &self.config.default_title
    }

    /// Get icon paths for a specific tab, falling back to default
    pub fn get_icons(&self, tab_id: Option<u64>) -> &HashMap<u32, String> {
        if let Some(tid) = tab_id {
            if let Some(icons) = self.tab_icons.get(&tid) {
                return icons;
            }
        }
        &self.config.default_icon
    }

    /// Get popup path for a specific tab, falling back to default
    pub fn get_popup(&self, tab_id: Option<u64>) -> Option<&str> {
        if let Some(tid) = tab_id {
            if let Some(popup) = self.tab_popups.get(&tid) {
                return Some(popup);
            }
        }
        self.config.default_popup.as_deref()
    }

    /// Get badge text for a specific tab, falling back to global
    pub fn get_badge_text(&self, tab_id: Option<u64>) -> &str {
        if let Some(tid) = tab_id {
            if let Some(text) = self.tab_badge_text.get(&tid) {
                return text;
            }
        }
        &self.badge_text
    }

    /// Set global badge text
    pub fn set_badge_text(&mut self, text: String) {
        self.badge_text = text;
    }

    /// Set tab-specific badge text
    pub fn set_tab_badge_text(&mut self, tab_id: u64, text: String) {
        self.tab_badge_text.insert(tab_id, text);
    }

    /// Set global title
    pub fn set_title(&mut self, title: String) {
        self.config.default_title = title;
    }

    /// Set tab-specific title
    pub fn set_tab_title(&mut self, tab_id: u64, title: String) {
        self.tab_titles.insert(tab_id, title);
    }

    /// Set tab-specific popup
    pub fn set_tab_popup(&mut self, tab_id: u64, popup: String) {
        self.tab_popups.insert(tab_id, popup);
    }

    /// Clear tab-specific state when tab is closed
    pub fn clear_tab_state(&mut self, tab_id: u64) {
        self.tab_titles.remove(&tab_id);
        self.tab_icons.remove(&tab_id);
        self.tab_popups.remove(&tab_id);
        self.tab_badge_text.remove(&tab_id);
    }
}

/// Browser Action API
///
/// Manages browser actions for all extensions
pub struct BrowserActionApi {
    /// Registered browser actions (extension_id -> state)
    actions: HashMap<ExtensionId, BrowserActionState>,
}

impl BrowserActionApi {
    /// Create a new BrowserActionApi
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Register a browser action for an extension
    pub fn register(&mut self, extension_id: ExtensionId, action: BrowserAction) {
        let state = BrowserActionState::from_config(action);
        self.actions.insert(extension_id, state);
    }

    /// Unregister a browser action
    pub fn unregister(&mut self, extension_id: ExtensionId) {
        self.actions.remove(&extension_id);
    }

    /// Get browser action state for an extension
    pub fn get(&self, extension_id: ExtensionId) -> Option<&BrowserActionState> {
        self.actions.get(&extension_id)
    }

    /// Get mutable browser action state for an extension
    pub fn get_mut(&mut self, extension_id: ExtensionId) -> Option<&mut BrowserActionState> {
        self.actions.get_mut(&extension_id)
    }

    /// Set badge text for an extension
    pub fn set_badge_text(&mut self, extension_id: ExtensionId, text: String) {
        if let Some(state) = self.actions.get_mut(&extension_id) {
            state.set_badge_text(text);
        }
    }

    /// Set badge text for a specific tab
    pub fn set_tab_badge_text(&mut self, extension_id: ExtensionId, tab_id: u64, text: String) {
        if let Some(state) = self.actions.get_mut(&extension_id) {
            state.set_tab_badge_text(tab_id, text);
        }
    }

    /// Set badge background color for an extension
    pub fn set_badge_background_color(&mut self, extension_id: ExtensionId, color: String) {
        if let Some(state) = self.actions.get_mut(&extension_id) {
            state.badge_background_color = color;
        }
    }

    /// Enable/disable browser action
    pub fn set_enabled(&mut self, extension_id: ExtensionId, enabled: bool) {
        if let Some(state) = self.actions.get_mut(&extension_id) {
            state.enabled = enabled;
        }
    }

    /// Get all registered extension IDs
    pub fn list_extensions(&self) -> Vec<ExtensionId> {
        self.actions.keys().cloned().collect()
    }

    /// Get actions that should be displayed in toolbar
    pub fn visible_actions(&self) -> Vec<(ExtensionId, &BrowserActionState)> {
        self.actions
            .iter()
            .filter(|(_, state)| state.enabled)
            .map(|(id, state)| (*id, state))
            .collect()
    }

    /// Notify when a tab is closed to clean up tab-specific state
    pub fn on_tab_closed(&mut self, tab_id: u64) {
        for state in self.actions.values_mut() {
            state.clear_tab_state(tab_id);
        }
    }
}

impl Default for BrowserActionApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_action_creation() {
        let action = BrowserAction::new("Test Action".to_string())
            .with_popup("popup.html".to_string())
            .with_icon(16, "icon16.png".to_string())
            .with_icon(48, "icon48.png".to_string());

        assert_eq!(action.default_title, "Test Action");
        assert_eq!(action.default_popup, Some("popup.html".to_string()));
        assert_eq!(action.default_icon.len(), 2);
    }

    #[test]
    fn test_browser_action_state() {
        let action = BrowserAction::new("Test".to_string());
        let mut state = BrowserActionState::from_config(action);

        assert!(state.enabled);
        assert_eq!(state.get_title(None), "Test");
        assert!(state.get_badge_text(None).is_empty());

        state.set_badge_text("5".to_string());
        assert_eq!(state.get_badge_text(None), "5");

        state.set_tab_badge_text(1, "10".to_string());
        assert_eq!(state.get_badge_text(Some(1)), "10");
        assert_eq!(state.get_badge_text(Some(2)), "5"); // Falls back to global
    }

    #[test]
    fn test_browser_action_api() {
        let mut api = BrowserActionApi::new();
        let ext_id = ExtensionId::from_string("test-ext");
        let action = BrowserAction::new("Test".to_string());

        api.register(ext_id, action);
        assert!(api.get(ext_id).is_some());

        api.set_badge_text(ext_id, "3".to_string());
        assert_eq!(api.get(ext_id).unwrap().badge_text, "3");

        api.unregister(ext_id);
        assert!(api.get(ext_id).is_none());
    }

    #[test]
    fn test_tab_state_cleanup() {
        let mut api = BrowserActionApi::new();
        let ext_id = ExtensionId::from_string("test-ext");
        let action = BrowserAction::new("Test".to_string());

        api.register(ext_id, action);
        api.set_tab_badge_text(ext_id, 1, "5".to_string());

        assert_eq!(api.get(ext_id).unwrap().get_badge_text(Some(1)), "5");

        api.on_tab_closed(1);
        assert_eq!(api.get(ext_id).unwrap().get_badge_text(Some(1)), ""); // Falls back to empty global
    }
}
