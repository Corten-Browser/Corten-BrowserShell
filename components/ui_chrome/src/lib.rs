//! UI Chrome Component
//!
//! Browser UI elements including address bar, toolbar, tab bar, and keyboard shortcuts.
//!
//! This component provides the browser chrome using egui for rendering the UI.

use shared_types::{ComponentError, KeyboardShortcut, TabId};
use std::collections::{HashMap, HashSet};

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

/// Types of context menus
#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuType {
    /// Context menu for a tab
    Tab(TabId),
    /// Context menu for the address bar
    AddressBar,
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
/// - UI panels (settings, history, downloads)
/// - Context menus
/// - Status bar
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

    /// Whether the settings panel is visible
    settings_panel_visible: bool,

    /// Whether the history panel is visible
    history_panel_visible: bool,

    /// Whether the downloads panel is visible
    downloads_panel_visible: bool,

    /// Active context menu, if any
    active_context_menu: Option<ContextMenuType>,

    /// URL being hovered over (for status bar display)
    hover_url: Option<String>,

    /// Number of active downloads
    download_count: usize,

    /// Bookmarked URLs
    bookmarks: HashSet<String>,
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
            settings_panel_visible: false,
            history_panel_visible: false,
            downloads_panel_visible: false,
            active_context_menu: None,
            hover_url: None,
            download_count: 0,
            bookmarks: HashSet::new(),
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
        let index = self
            .tab_order
            .iter()
            .position(|&id| id == tab_id)
            .ok_or_else(|| {
                ComponentError::ResourceNotFound(format!("Tab {:?} not found", tab_id))
            })?;

        self.active_tab_index = index;
        Ok(())
    }

    /// Update a tab's title
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::ResourceNotFound` if the tab doesn't exist
    pub fn update_tab_title(&mut self, tab_id: TabId, title: String) -> Result<(), ComponentError> {
        let tab = self.tabs.get_mut(&tab_id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!("Tab {:?} not found", tab_id))
        })?;

        tab.title = title;
        Ok(())
    }

    /// Update a tab's loading state
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::ResourceNotFound` if the tab doesn't exist
    pub fn update_loading_state(
        &mut self,
        tab_id: TabId,
        loading: bool,
    ) -> Result<(), ComponentError> {
        let tab = self.tabs.get_mut(&tab_id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!("Tab {:?} not found", tab_id))
        })?;

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
    pub fn handle_keyboard_shortcut(
        &mut self,
        shortcut: KeyboardShortcut,
    ) -> Result<(), ComponentError> {
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
                        "Cannot close the last tab".to_string(),
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

    // Phase 3 Enhancement Methods

    /// Close a specific tab by ID
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::InvalidState` if attempting to close the last tab
    /// Returns `ComponentError::ResourceNotFound` if the tab doesn't exist
    pub fn close_tab(&mut self, tab_id: TabId) -> Result<(), ComponentError> {
        // Check if this is the last tab
        if self.tab_count() <= 1 {
            return Err(ComponentError::InvalidState(
                "Cannot close the last tab".to_string(),
            ));
        }

        // Find the tab in tab_order
        let position = self
            .tab_order
            .iter()
            .position(|&id| id == tab_id)
            .ok_or_else(|| {
                ComponentError::ResourceNotFound(format!("Tab {:?} not found", tab_id))
            })?;

        // Remove from tabs map
        self.tabs.remove(&tab_id);

        // Remove from tab order
        self.tab_order.remove(position);

        // Adjust active index if needed
        if position <= self.active_tab_index && self.active_tab_index > 0 {
            self.active_tab_index -= 1;
        } else if self.active_tab_index >= self.tab_order.len() {
            self.active_tab_index = self.tab_order.len() - 1;
        }

        Ok(())
    }

    /// Switch to the next tab (wraps around)
    pub fn switch_to_next_tab(&mut self) -> Result<(), ComponentError> {
        if self.tab_order.is_empty() {
            return Err(ComponentError::InvalidState(
                "No tabs available".to_string(),
            ));
        }

        self.active_tab_index = (self.active_tab_index + 1) % self.tab_order.len();
        Ok(())
    }

    /// Switch to the previous tab (wraps around)
    pub fn switch_to_previous_tab(&mut self) -> Result<(), ComponentError> {
        if self.tab_order.is_empty() {
            return Err(ComponentError::InvalidState(
                "No tabs available".to_string(),
            ));
        }

        if self.active_tab_index == 0 {
            self.active_tab_index = self.tab_order.len() - 1;
        } else {
            self.active_tab_index -= 1;
        }
        Ok(())
    }

    /// Switch to a specific tab by number (1-9)
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::InvalidState` if the tab number is out of range
    pub fn switch_to_tab_number(&mut self, tab_number: usize) -> Result<(), ComponentError> {
        if tab_number == 0 || tab_number > self.tab_order.len() {
            return Err(ComponentError::InvalidState(format!(
                "Tab number {} is out of range (have {} tabs)",
                tab_number,
                self.tab_order.len()
            )));
        }

        self.active_tab_index = tab_number - 1; // Convert to 0-indexed
        Ok(())
    }

    /// Check if settings panel is visible
    pub fn is_settings_panel_visible(&self) -> bool {
        self.settings_panel_visible
    }

    /// Check if history panel is visible
    pub fn is_history_panel_visible(&self) -> bool {
        self.history_panel_visible
    }

    /// Check if downloads panel is visible
    pub fn is_downloads_panel_visible(&self) -> bool {
        self.downloads_panel_visible
    }

    /// Toggle settings panel visibility
    pub fn toggle_settings_panel(&mut self) {
        self.settings_panel_visible = !self.settings_panel_visible;
    }

    /// Toggle history panel visibility
    pub fn toggle_history_panel(&mut self) {
        self.history_panel_visible = !self.history_panel_visible;
    }

    /// Toggle downloads panel visibility
    pub fn toggle_downloads_panel(&mut self) {
        self.downloads_panel_visible = !self.downloads_panel_visible;
    }

    /// Check if there is an active context menu
    pub fn has_active_context_menu(&self) -> bool {
        self.active_context_menu.is_some()
    }

    /// Show context menu for a tab
    pub fn show_tab_context_menu(&mut self, tab_id: TabId) {
        self.active_context_menu = Some(ContextMenuType::Tab(tab_id));
    }

    /// Show context menu for address bar
    pub fn show_address_bar_context_menu(&mut self) {
        self.active_context_menu = Some(ContextMenuType::AddressBar);
    }

    /// Close the active context menu
    pub fn close_context_menu(&mut self) {
        self.active_context_menu = None;
    }

    /// Set the hover URL for status bar display
    pub fn set_hover_url(&mut self, url: Option<String>) {
        self.hover_url = url;
    }

    /// Get the current hover URL
    pub fn get_hover_url(&self) -> Option<String> {
        self.hover_url.clone()
    }

    /// Set the download count
    pub fn set_download_count(&mut self, count: usize) {
        self.download_count = count;
    }

    /// Get the current download count
    pub fn get_download_count(&self) -> usize {
        self.download_count
    }

    /// Bookmark the current page
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::InvalidState` if the address bar is empty
    pub fn bookmark_current_page(&mut self) -> Result<(), ComponentError> {
        if self.address_bar_text.is_empty() {
            return Err(ComponentError::InvalidState(
                "Cannot bookmark an empty address".to_string(),
            ));
        }

        self.bookmarks.insert(self.address_bar_text.clone());
        Ok(())
    }

    /// Check if a URL is bookmarked
    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.contains(url)
    }

    /// Render the UI chrome using egui
    ///
    /// # Errors
    ///
    /// Returns `ComponentError` if rendering fails
    pub fn render(&mut self, ctx: &egui::Context) -> Result<(), ComponentError> {
        // Handle keyboard shortcuts
        self.handle_keyboard_input(ctx);

        // Top toolbar with navigation buttons
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("◀").clicked() {
                    // Go back - would send message via message bus
                }
                if ui.button("▶").clicked() {
                    // Go forward - would send message via message bus
                }
                if ui.button("⟳").clicked() {
                    // Reload - would send message via message bus
                }

                // Address bar with context menu support
                let mut address_text = self.address_bar_text.clone();
                let response = ui.text_edit_singleline(&mut address_text);

                if response.changed() {
                    self.address_bar_text = address_text;
                }

                // Track focus state
                self.address_bar_focused = response.has_focus();

                // Right-click context menu
                if response.secondary_clicked() {
                    self.show_address_bar_context_menu();
                }

                if ui.button("Go").clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    // Navigate - would send message via message bus
                }
            });
        });

        // Tab bar with close buttons
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut tab_to_close: Option<TabId> = None;
                let mut tab_for_context_menu: Option<TabId> = None;

                // Render tabs in order
                for (index, &tab_id) in self.tab_order.iter().enumerate() {
                    if let Some(tab) = self.tabs.get(&tab_id) {
                        let is_active = index == self.active_tab_index;

                        // Show loading indicator if tab is loading
                        let label = if tab.loading {
                            format!("⟳ {}", tab.title)
                        } else {
                            tab.title.clone()
                        };

                        // Tab with hover effect
                        let tab_response = ui.selectable_label(is_active, &label);

                        if tab_response.clicked() {
                            self.active_tab_index = index;
                        }

                        // Middle-click to close
                        if tab_response.middle_clicked() {
                            tab_to_close = Some(tab_id);
                        }

                        // Right-click context menu (deferred to avoid borrow issues)
                        if tab_response.secondary_clicked() {
                            tab_for_context_menu = Some(tab_id);
                        }

                        // Close button (X)
                        if ui.small_button("✕").clicked() {
                            tab_to_close = Some(tab_id);
                        }
                    }
                }

                // Process deferred actions
                if let Some(tab_id) = tab_to_close {
                    let _ = self.close_tab(tab_id);
                }

                if let Some(tab_id) = tab_for_context_menu {
                    self.show_tab_context_menu(tab_id);
                }

                // New tab button
                if ui.button("+").clicked() {
                    self.add_tab("New Tab".to_string());
                }
            });
        });

        // Render context menus
        self.render_context_menu(ctx);

        // Left side panels (Settings, History, Downloads)
        if self.settings_panel_visible {
            egui::SidePanel::left("settings_panel")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("⚙ Settings");
                    ui.separator();
                    ui.label("Theme: Dark");
                    ui.label("Download location: ~/Downloads");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.toggle_settings_panel();
                    }
                });
        }

        if self.history_panel_visible {
            egui::SidePanel::left("history_panel")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("📜 History");
                    ui.separator();
                    ui.label("No history yet");
                    ui.separator();
                    if ui.button("Clear History").clicked() {
                        // Clear history - would send message via message bus
                    }
                    if ui.button("Close").clicked() {
                        self.toggle_history_panel();
                    }
                });
        }

        if self.downloads_panel_visible {
            egui::SidePanel::left("downloads_panel")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("⬇ Downloads");
                    ui.separator();
                    if self.download_count > 0 {
                        ui.label(format!("Active downloads: {}", self.download_count));
                    } else {
                        ui.label("No active downloads");
                    }
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.toggle_downloads_panel();
                    }
                });
        }

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Loading status for active tab
                if let Some(tab_id) = self.active_tab_id() {
                    if let Some(loading) = self.is_tab_loading(tab_id) {
                        if loading {
                            ui.label("⟳ Loading...");
                        } else {
                            ui.label("✓ Ready");
                        }
                    }
                }

                ui.separator();

                // Hover URL display
                if let Some(url) = &self.hover_url {
                    ui.label(url);
                } else {
                    ui.label("");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Download count badge
                    if self.download_count > 0 {
                        ui.label(format!("⬇ {}", self.download_count));
                    }
                });
            });
        });

        // Central panel (placeholder for web content)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Web Content Area");
                ui.label("(WebView will be embedded here)");

                ui.separator();
                ui.label("Keyboard Shortcuts:");
                ui.label("Ctrl+T: New tab");
                ui.label("Ctrl+W: Close tab");
                ui.label("Ctrl+Tab: Next tab");
                ui.label("Ctrl+Shift+Tab: Previous tab");
                ui.label("Ctrl+1-9: Switch to tab N");
                ui.label("Ctrl+D: Bookmark page");
                ui.label("Ctrl+H: History panel");
                ui.label("Ctrl+J: Downloads panel");
                ui.label("Ctrl+,: Settings panel");
            });
        });

        Ok(())
    }

    /// Handle keyboard input for shortcuts
    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl;
            let shift = i.modifiers.shift;

            // Ctrl+Tab: Next tab
            if ctrl && !shift && i.key_pressed(egui::Key::Tab) {
                let _ = self.switch_to_next_tab();
            }

            // Ctrl+Shift+Tab: Previous tab
            if ctrl && shift && i.key_pressed(egui::Key::Tab) {
                let _ = self.switch_to_previous_tab();
            }

            // Ctrl+1-9: Switch to tab number
            for (key, num) in [
                (egui::Key::Num1, 1),
                (egui::Key::Num2, 2),
                (egui::Key::Num3, 3),
                (egui::Key::Num4, 4),
                (egui::Key::Num5, 5),
                (egui::Key::Num6, 6),
                (egui::Key::Num7, 7),
                (egui::Key::Num8, 8),
                (egui::Key::Num9, 9),
            ] {
                if ctrl && i.key_pressed(key) {
                    let _ = self.switch_to_tab_number(num);
                }
            }

            // Ctrl+D: Bookmark current page
            if ctrl && i.key_pressed(egui::Key::D) {
                let _ = self.bookmark_current_page();
            }

            // Ctrl+H: Toggle history panel
            if ctrl && i.key_pressed(egui::Key::H) {
                self.toggle_history_panel();
            }

            // Ctrl+J: Toggle downloads panel
            if ctrl && i.key_pressed(egui::Key::J) {
                self.toggle_downloads_panel();
            }

            // Ctrl+,: Toggle settings panel
            if ctrl && i.key_pressed(egui::Key::Comma) {
                self.toggle_settings_panel();
            }
        });
    }

    /// Render context menus
    fn render_context_menu(&mut self, ctx: &egui::Context) {
        if let Some(menu_type) = self.active_context_menu.clone() {
            match menu_type {
                ContextMenuType::Tab(tab_id) => {
                    egui::Area::new(egui::Id::new("tab_context_menu"))
                        .fixed_pos(ctx.pointer_latest_pos().unwrap_or_default())
                        .show(ctx, |ui| {
                            egui::Frame::menu(ui.style()).show(ui, |ui| {
                                if ui.button("Close Tab").clicked() {
                                    let _ = self.close_tab(tab_id);
                                    self.close_context_menu();
                                }
                                if ui.button("Close Other Tabs").clicked() {
                                    // Close all tabs except this one
                                    self.close_context_menu();
                                }
                                if ui.button("Close All Tabs").clicked() {
                                    // Close all tabs (except last one)
                                    self.close_context_menu();
                                }
                            });
                        });

                    // Close menu on any click outside
                    if ctx.input(|i| i.pointer.any_click()) {
                        self.close_context_menu();
                    }
                }
                ContextMenuType::AddressBar => {
                    egui::Area::new(egui::Id::new("address_bar_context_menu"))
                        .fixed_pos(ctx.pointer_latest_pos().unwrap_or_default())
                        .show(ctx, |ui| {
                            egui::Frame::menu(ui.style()).show(ui, |ui| {
                                if ui.button("Copy").clicked() {
                                    // Copy address bar text to clipboard
                                    self.close_context_menu();
                                }
                                if ui.button("Paste").clicked() {
                                    // Paste from clipboard to address bar
                                    self.close_context_menu();
                                }
                            });
                        });

                    // Close menu on any click outside
                    if ctx.input(|i| i.pointer.any_click()) {
                        self.close_context_menu();
                    }
                }
            }
        }
    }
}

impl Default for UiChrome {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement eframe::App trait to enable egui rendering
impl eframe::App for UiChrome {
    /// Update and render the UI chrome
    ///
    /// This is called by eframe on each frame to update and render the UI.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Call the existing render method which handles all the UI logic
        // Ignore any errors from render() as we can't propagate them from update()
        let _ = self.render(ctx);
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
