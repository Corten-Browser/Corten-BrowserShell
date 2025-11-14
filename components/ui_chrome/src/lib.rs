//! UI Chrome Component
//!
//! Browser UI elements including address bar, toolbar, tab bar, and keyboard shortcuts.
//!
//! This component provides the browser chrome using egui for rendering the UI.

use shared_types::{ComponentError, KeyboardShortcut, TabId};
use std::collections::HashMap;

/// State for a single tab
#[derive(Debug, Clone)]
pub struct TabState {
    /// Unique identifier for this tab
    pub id: TabId,

    /// Display title for the tab
    pub title: String,

    /// Whether the tab is currently loading
    pub loading: bool,
}

impl TabState {
    /// Create a new tab with default state
    pub fn new(title: String) -> Self {
        Self {
            id: TabId::new(),
            title,
            loading: false,
        }
    }

    /// Create a tab with a specific ID (for testing)
    pub fn with_id(id: TabId, title: String) -> Self {
        Self {
            id,
            title,
            loading: false,
        }
    }
}

/// Browser UI chrome component
///
/// Manages the browser's UI elements including:
/// - Address bar
/// - Navigation toolbar (back/forward/reload)
/// - Tab bar with tab management
/// - Keyboard shortcuts
pub struct UiChrome {
    /// Current text in the address bar
    address_bar_text: String,

    /// All tabs, indexed by TabId
    tabs: HashMap<TabId, TabState>,

    /// Ordered list of tab IDs (for tab bar rendering order)
    tab_order: Vec<TabId>,

    /// Index of the currently active tab in tab_order
    active_tab_index: usize,

    /// Whether the address bar has focus
    address_bar_focused: bool,
}

impl UiChrome {
    /// Create a new UiChrome instance with one default tab
    pub fn new() -> Self {
        let default_tab = TabState::new("New Tab".to_string());
        let tab_id = default_tab.id;

        let mut tabs = HashMap::new();
        tabs.insert(tab_id, default_tab);

        Self {
            address_bar_text: String::new(),
            tabs,
            tab_order: vec![tab_id],
            active_tab_index: 0,
            address_bar_focused: false,
        }
    }

    /// Get the current address bar text
    pub fn address_bar_text(&self) -> &str {
        &self.address_bar_text
    }

    /// Get the number of tabs
    pub fn tab_count(&self) -> usize {
        self.tab_order.len()
    }

    /// Get the active tab index
    pub fn active_tab_index(&self) -> usize {
        self.active_tab_index
    }

    /// Get a tab ID by its position index
    pub fn get_tab_id(&self, index: usize) -> Option<TabId> {
        self.tab_order.get(index).copied()
    }

    /// Get the title of a tab
    pub fn get_tab_title(&self, tab_id: TabId) -> Option<String> {
        self.tabs.get(&tab_id).map(|t| t.title.clone())
    }

    /// Check if a tab is loading
    pub fn is_tab_loading(&self, tab_id: TabId) -> Option<bool> {
        self.tabs.get(&tab_id).map(|t| t.loading)
    }

    /// Get the currently active tab ID
    pub fn active_tab_id(&self) -> Option<TabId> {
        self.tab_order.get(self.active_tab_index).copied()
    }

    /// Check if the address bar is focused
    pub fn is_address_bar_focused(&self) -> bool {
        self.address_bar_focused
    }

    /// Add a new tab with the given title
    pub fn add_tab(&mut self, title: String) -> TabId {
        let tab = TabState::new(title);
        let tab_id = tab.id;

        self.tabs.insert(tab_id, tab);
        self.tab_order.push(tab_id);

        // Set the new tab as active
        self.active_tab_index = self.tab_order.len() - 1;

        tab_id
    }

    /// Set the active tab by ID
    pub fn set_active_tab(&mut self, tab_id: TabId) -> Result<(), ComponentError> {
        // Find the index of this tab
        let index = self.tab_order.iter()
            .position(|&id| id == tab_id)
            .ok_or_else(|| ComponentError::ResourceNotFound(
                format!("Tab {:?} not found", tab_id)
            ))?;

        self.active_tab_index = index;
        Ok(())
    }

    /// Update a tab's title
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::ResourceNotFound` if the tab doesn't exist
    pub fn update_tab_title(&mut self, tab_id: TabId, title: String) -> Result<(), ComponentError> {
        let tab = self.tabs.get_mut(&tab_id)
            .ok_or_else(|| ComponentError::ResourceNotFound(
                format!("Tab {:?} not found", tab_id)
            ))?;

        tab.title = title;
        Ok(())
    }

    /// Update a tab's loading state
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::ResourceNotFound` if the tab doesn't exist
    pub fn update_loading_state(&mut self, tab_id: TabId, loading: bool) -> Result<(), ComponentError> {
        let tab = self.tabs.get_mut(&tab_id)
            .ok_or_else(|| ComponentError::ResourceNotFound(
                format!("Tab {:?} not found", tab_id)
            ))?;

        tab.loading = loading;
        Ok(())
    }

    /// Handle address bar input
    ///
    /// # Errors
    ///
    /// Returns `ComponentError` if the input is invalid
    pub fn handle_address_bar_input(&mut self, text: String) -> Result<(), ComponentError> {
        self.address_bar_text = text;
        Ok(())
    }

    /// Handle keyboard shortcuts
    ///
    /// # Errors
    ///
    /// Returns `ComponentError` if the shortcut cannot be handled
    pub fn handle_keyboard_shortcut(&mut self, shortcut: KeyboardShortcut) -> Result<(), ComponentError> {
        match shortcut {
            KeyboardShortcut::CtrlT => {
                // New tab
                self.add_tab("New Tab".to_string());
                Ok(())
            }

            KeyboardShortcut::CtrlW => {
                // Close tab - but not if it's the last one
                if self.tab_count() <= 1 {
                    return Err(ComponentError::InvalidState(
                        "Cannot close the last tab".to_string()
                    ));
                }

                // Remove the active tab
                if let Some(tab_id) = self.active_tab_id() {
                    // Remove from tabs map
                    self.tabs.remove(&tab_id);

                    // Remove from tab order
                    self.tab_order.remove(self.active_tab_index);

                    // Adjust active index if needed
                    if self.active_tab_index >= self.tab_order.len() && self.active_tab_index > 0 {
                        self.active_tab_index = self.tab_order.len() - 1;
                    }
                }

                Ok(())
            }

            KeyboardShortcut::CtrlL => {
                // Focus address bar
                self.address_bar_focused = true;
                Ok(())
            }

            KeyboardShortcut::F5 | KeyboardShortcut::CtrlR => {
                // Reload - would send message to active tab via message bus
                // For now, just acknowledge the shortcut
                Ok(())
            }

            _ => {
                // Other shortcuts not yet implemented
                Ok(())
            }
        }
    }

    /// Render the UI chrome using egui
    ///
    /// # Errors
    ///
    /// Returns `ComponentError` if rendering fails
    pub fn render(&mut self, ctx: &egui::Context) -> Result<(), ComponentError> {
        // Top toolbar with navigation buttons
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("<").clicked() {
                    // Go back - would send message via message bus
                }
                if ui.button(">").clicked() {
                    // Go forward - would send message via message bus
                }
                if ui.button("R").clicked() {
                    // Reload - would send message via message bus
                }

                // Address bar
                let mut address_text = self.address_bar_text.clone();
                let response = ui.text_edit_singleline(&mut address_text);

                if response.changed() {
                    self.address_bar_text = address_text;
                }

                // Track focus state
                self.address_bar_focused = response.has_focus();

                if ui.button("Go").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    // Navigate - would send message via message bus
                }
            });
        });

        // Tab bar
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Render tabs in order
                for (index, &tab_id) in self.tab_order.iter().enumerate() {
                    if let Some(tab) = self.tabs.get(&tab_id) {
                        let is_active = index == self.active_tab_index;

                        // Show loading indicator if tab is loading
                        let label = if tab.loading {
                            format!("[L] {}", tab.title)
                        } else {
                            tab.title.clone()
                        };

                        if ui.selectable_label(is_active, &label).clicked() {
                            self.active_tab_index = index;
                        }
                    }
                }

                // New tab button
                if ui.button("+").clicked() {
                    self.add_tab("New Tab".to_string());
                }
            });
        });

        // Central panel (placeholder for web content)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Web Content Area");
                ui.label("(WebView will be embedded here)");
            });
        });

        Ok(())
    }
}

impl Default for UiChrome {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_state_creation() {
        let tab = TabState::new("Test Tab".to_string());
        assert_eq!(tab.title, "Test Tab");
        assert_eq!(tab.loading, false);
    }

    #[test]
    fn test_ui_chrome_default() {
        let chrome = UiChrome::default();
        assert_eq!(chrome.tab_count(), 1);
    }
}
