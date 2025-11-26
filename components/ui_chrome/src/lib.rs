//! UI Chrome Component
//!
//! Browser UI elements including address bar, toolbar, tab bar, and keyboard shortcuts.
//!
//! This component provides the browser chrome using egui for rendering the UI.
//!
//! # Theme System
//!
//! The UI chrome includes a theme system supporting light, dark, and auto modes:
//!
//! ```rust,ignore
//! use ui_chrome::theme::{Theme, ThemeMode, ThemeManager};
//!
//! // Create a theme manager
//! let mut theme_manager = ThemeManager::new();
//!
//! // Switch to dark mode
//! theme_manager.set_mode(ThemeMode::Dark);
//!
//! // Apply to egui context
//! theme_manager.apply(&ctx);
//! ```
//!
//! # Developer Tools
//!
//! The component includes a Chrome-style developer tools panel:
//!
//! ```rust,ignore
//! use ui_chrome::devtools::{DevToolsPanel, DevToolsConfig, DockPosition};
//!
//! // Create dev tools panel
//! let mut devtools = DevToolsPanel::default();
//!
//! // Toggle visibility with F12 or menu
//! devtools.toggle();
//!
//! // Log to console
//! devtools.console_log("Hello from devtools!");
//! devtools.console_error("An error occurred");
//!
//! // Track network requests
//! let id = devtools.add_network_request(HttpMethod::GET, "https://example.com/api");
//! devtools.complete_network_request(id, 200, Some("application/json".to_string()));
//!
//! // Render the panel
//! devtools.show(&ctx);
//! ```
//!
//! # Print Support
//!
//! The UI chrome includes print support with preview, settings, and job tracking:
//!
//! ```rust,ignore
//! use ui_chrome::print::{PrintManager, PrintSettings, PaperSize, Orientation};
//!
//! // Create print manager
//! let mut print_manager = PrintManager::new();
//!
//! // Configure settings
//! let settings = PrintSettings::default()
//!     .with_paper_size(PaperSize::A4)
//!     .with_orientation(Orientation::Portrait)
//!     .with_scale(100);
//!
//! // Show print preview
//! print_manager.show_preview_with_settings(settings);
//!
//! // Create a print job
//! let job_id = print_manager.create_job("Document.pdf".to_string(), 10);
//! ```

pub mod crash_recovery;
pub mod devtools;
pub mod menu;
pub mod print;
pub mod settings_ui;
pub mod tab_drag_ui;
pub mod theme;

use crash_recovery::{ClosedTabInfo, CrashRecoveryUi};
use shared_types::{ComponentError, DownloadId, KeyboardShortcut, TabId};
use std::collections::{HashMap, HashSet};
use tab_drag_ui::{TabDragState, TabDragVisuals, TabOverflowHandler};

// Re-export theme types for convenience
pub use theme::{Theme, ThemeManager, ThemeMode};

// Re-export devtools types for convenience
pub use devtools::{
    ConsoleLevel, ConsoleMessage, DevToolsConfig, DevToolsPanel, DevToolsState, DevToolsTab,
    DockPosition, HttpMethod, NetworkInspectorEntry, NetworkStatus, NetworkTiming,
};

// Re-export print types for convenience
pub use print::{
    Orientation, PageRange, PaperSize, PrintDestination, PrintError, PrintJob, PrintJobId,
    PrintJobStatus, PrintManager, PrintManagerResponse, PrintMargins, PrintPreview,
    PrintPreviewResponse, PrintQuality, PrintSettings,
};

// Re-export menu types for convenience
pub use menu::{MenuAction, MenuBar, PanelType, UiAction};

// Re-export settings UI types for convenience
pub use settings_ui::{SettingsTab, SettingsUi};

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

/// Download status for UI display
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadDisplayStatus {
    /// Download is in progress
    Downloading,
    /// Download is paused
    Paused,
    /// Download completed successfully
    Complete,
    /// Download failed
    Failed(String),
}

/// Download information for UI display
#[derive(Debug, Clone)]
pub struct DownloadDisplay {
    /// Unique identifier
    pub id: DownloadId,
    /// Filename
    pub filename: String,
    /// Downloaded bytes
    pub downloaded_bytes: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Download speed in bytes/sec
    pub bytes_per_second: u64,
    /// ETA in seconds
    pub eta_seconds: u64,
    /// Status
    pub status: DownloadDisplayStatus,
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
/// - Application menu bar
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

    /// List of downloads for display
    downloads: Vec<DownloadDisplay>,

    /// Bookmarked URLs
    bookmarks: HashSet<String>,

    /// Application menu bar
    menu_bar: MenuBar,

    /// Settings UI panel
    settings_ui: SettingsUi,

    /// Tab drag and drop state
    tab_drag_state: TabDragState,

    /// Tab drag visual configuration
    tab_drag_visuals: TabDragVisuals,

    /// Tab overflow handler for scrolling
    tab_overflow: TabOverflowHandler,

    /// Blocked content count from ad blocker (for status bar display)
    blocked_content_count: usize,

    /// Crash recovery UI (session restore dialog and recently closed tabs)
    crash_recovery: CrashRecoveryUi,
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
            downloads: Vec::new(),
            bookmarks: HashSet::new(),
            menu_bar: MenuBar::new(),
            settings_ui: SettingsUi::new(),
            tab_drag_state: TabDragState::new(),
            tab_drag_visuals: TabDragVisuals::default(),
            tab_overflow: TabOverflowHandler::new(),
            blocked_content_count: 0,
            crash_recovery: CrashRecoveryUi::new(),
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

        // Get tab info before removing for recently closed tracking
        if let Some(tab) = self.tabs.get(&tab_id) {
            let closed_tab = ClosedTabInfo {
                id: tab_id,
                title: tab.title.clone(),
                url: self.address_bar_text.clone(), // In real usage, should track per-tab URLs
                closed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            self.crash_recovery.add_closed_tab(closed_tab);
        }

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

    /// Update the downloads list for display
    pub fn set_downloads(&mut self, downloads: Vec<DownloadDisplay>) {
        self.download_count = downloads
            .iter()
            .filter(|d| {
                matches!(
                    d.status,
                    DownloadDisplayStatus::Downloading | DownloadDisplayStatus::Paused
                )
            })
            .count();
        self.downloads = downloads;
    }

    /// Get the current downloads list
    pub fn get_downloads(&self) -> &[DownloadDisplay] {
        &self.downloads
    }

    /// Clear completed downloads from the list
    pub fn clear_completed_downloads(&mut self) {
        self.downloads
            .retain(|d| !matches!(d.status, DownloadDisplayStatus::Complete));
        self.download_count = self
            .downloads
            .iter()
            .filter(|d| {
                matches!(
                    d.status,
                    DownloadDisplayStatus::Downloading | DownloadDisplayStatus::Paused
                )
            })
            .count();
    }

    /// Format file size in human-readable format (B, KB, MB, GB)
    fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Format download speed in human-readable format (KB/s, MB/s)
    fn format_speed(bytes_per_second: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;

        if bytes_per_second >= MB {
            format!("{:.1} MB/s", bytes_per_second as f64 / MB as f64)
        } else if bytes_per_second >= KB {
            format!("{:.1} KB/s", bytes_per_second as f64 / KB as f64)
        } else if bytes_per_second > 0 {
            format!("{} B/s", bytes_per_second)
        } else {
            String::from("-- KB/s")
        }
    }

    /// Format time duration in human-readable format (seconds, minutes, hours)
    fn format_time(seconds: u64) -> String {
        const MINUTE: u64 = 60;
        const HOUR: u64 = MINUTE * 60;

        if seconds >= HOUR {
            let hours = seconds / HOUR;
            let minutes = (seconds % HOUR) / MINUTE;
            if minutes > 0 {
                format!("{}h {}m", hours, minutes)
            } else {
                format!("{}h", hours)
            }
        } else if seconds >= MINUTE {
            let minutes = seconds / MINUTE;
            let secs = seconds % MINUTE;
            if secs > 0 {
                format!("{}m {}s", minutes, secs)
            } else {
                format!("{}m", minutes)
            }
        } else if seconds > 0 {
            format!("{}s", seconds)
        } else {
            String::from("--")
        }
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

    /// Set the blocked content count from ad blocker
    pub fn set_blocked_content_count(&mut self, count: usize) {
        self.blocked_content_count = count;
    }

    /// Get the current blocked content count
    pub fn get_blocked_content_count(&self) -> usize {
        self.blocked_content_count
    }

    /// Show the session restore dialog (call this at startup after crash detection)
    pub fn show_crash_recovery_dialog(&mut self) {
        self.crash_recovery.show_restore_dialog();
    }

    /// Check if user chose to restore session
    pub fn should_restore_session(&self) -> bool {
        self.crash_recovery.should_restore_session()
    }

    /// Check if restore dialog was dismissed
    pub fn restore_dialog_dismissed(&self) -> bool {
        self.crash_recovery.restore_dialog_dismissed()
    }

    /// Reset crash recovery dialog state after handling the choice
    pub fn reset_crash_recovery_dialog(&mut self) {
        self.crash_recovery.reset_restore_dialog();
    }

    /// Toggle the recently closed tabs menu
    pub fn toggle_recently_closed_tabs(&mut self) {
        self.crash_recovery.toggle_recently_closed_menu();
    }

    /// Get count of recently closed tabs
    pub fn recently_closed_count(&self) -> usize {
        self.crash_recovery.closed_tab_count()
    }

    /// Reorder tabs based on drag-and-drop operation
    ///
    /// Moves a tab from `from_index` to `to_index` in the tab order
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::InvalidState` if indices are out of range
    pub fn reorder_tab(&mut self, from_index: usize, to_index: usize) -> Result<(), ComponentError> {
        if from_index >= self.tab_order.len() {
            return Err(ComponentError::InvalidState(format!(
                "Source index {} is out of range (have {} tabs)",
                from_index,
                self.tab_order.len()
            )));
        }

        if to_index > self.tab_order.len() {
            return Err(ComponentError::InvalidState(format!(
                "Target index {} is out of range (have {} tabs)",
                to_index,
                self.tab_order.len()
            )));
        }

        if from_index == to_index {
            return Ok(()); // No-op
        }

        // Remove tab from original position
        let tab_id = self.tab_order.remove(from_index);

        // Insert at new position (adjust for removal)
        let insert_index = if to_index > from_index {
            to_index - 1
        } else {
            to_index
        };

        self.tab_order.insert(insert_index, tab_id);

        // Update active index if needed
        if self.active_tab_index == from_index {
            self.active_tab_index = insert_index;
        } else if from_index < self.active_tab_index && insert_index >= self.active_tab_index {
            self.active_tab_index -= 1;
        } else if from_index > self.active_tab_index && insert_index <= self.active_tab_index {
            self.active_tab_index += 1;
        }

        Ok(())
    }

    /// Load settings from a settings manager
    ///
    /// This should be called when initializing the UI or when settings need to be refreshed
    pub async fn load_settings(
        &mut self,
        settings_manager: &settings_manager::SettingsManager,
    ) -> Result<(), ComponentError> {
        let settings = settings_manager.get_all_settings().await?;
        self.settings_ui.load_settings(settings);
        Ok(())
    }

    /// Save settings to a settings manager
    ///
    /// This should be called when the user clicks "Save" in the settings UI
    pub async fn save_settings(
        &mut self,
        settings_manager: &settings_manager::SettingsManager,
    ) -> Result<(), ComponentError> {
        // Get all settings from the UI
        let settings = self.settings_ui.get_all_settings();

        // Save each setting
        for (key, value) in settings {
            settings_manager
                .set_setting(key.clone(), value.clone())
                .await?;
        }

        // Persist to disk
        settings_manager.save().await?;

        // Mark as saved in the UI
        self.settings_ui.mark_saved();

        Ok(())
    }

    /// Check if there are unsaved settings changes
    pub fn has_unsaved_settings(&self) -> bool {
        self.settings_ui.has_unsaved_changes
    }

    /// Handle menu action
    fn handle_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::TogglePanel(panel_type) => {
                match panel_type {
                    PanelType::Settings => self.toggle_settings_panel(),
                    PanelType::History => self.toggle_history_panel(),
                    PanelType::Downloads => self.toggle_downloads_panel(),
                    PanelType::Bookmarks => {
                        // TODO: Implement bookmarks panel
                    }
                    PanelType::DevTools => {
                        // TODO: Toggle devtools panel
                    }
                }
            }
            MenuAction::UiAction(ui_action) => {
                match ui_action {
                    UiAction::BookmarkPage => {
                        let _ = self.bookmark_current_page();
                    }
                    UiAction::ShowAllHistory => {
                        self.toggle_history_panel();
                    }
                    UiAction::ShowAllBookmarks => {
                        // TODO: Implement show all bookmarks
                    }
                    UiAction::ZoomIn => {
                        // TODO: Implement zoom in
                        let current = self.menu_bar.zoom_level();
                        self.menu_bar.set_zoom_level((current + 10).min(300));
                    }
                    UiAction::ZoomOut => {
                        // TODO: Implement zoom out
                        let current = self.menu_bar.zoom_level();
                        self.menu_bar.set_zoom_level((current.saturating_sub(10)).max(25));
                    }
                    UiAction::ResetZoom => {
                        // TODO: Implement reset zoom
                        self.menu_bar.set_zoom_level(100);
                    }
                    UiAction::FullScreen => {
                        // TODO: Implement full screen toggle
                    }
                    UiAction::Find => {
                        // TODO: Implement find dialog
                    }
                    UiAction::About => {
                        // TODO: Implement about dialog
                    }
                    UiAction::Undo | UiAction::Redo | UiAction::Cut | UiAction::Copy
                    | UiAction::Paste | UiAction::SelectAll => {
                        // TODO: Implement clipboard operations
                    }
                    UiAction::ClearHistory => {
                        // TODO: Implement clear history
                    }
                    UiAction::ReportIssue => {
                        // TODO: Open issue tracker
                    }
                    UiAction::ShowDocumentation => {
                        // TODO: Open documentation
                    }
                }
            }
            MenuAction::SendMessage(_msg) => {
                // TODO: Send message to message bus
            }
            MenuAction::None => {}
        }
    }

    /// Render the UI chrome using egui
    ///
    /// # Errors
    ///
    /// Returns `ComponentError` if rendering fails
    pub fn render(&mut self, ctx: &egui::Context) -> Result<(), ComponentError> {
        // Handle keyboard shortcuts
        self.handle_keyboard_input(ctx);

        // Update menu bar state based on current UI state
        self.menu_bar.set_tab_state(
            !self.tabs.is_empty(),
            self.tabs.len() > 1,
        );
        // TODO: Update navigation state from browser history
        // TODO: Update edit state from clipboard/undo manager

        // Menu bar at the very top
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            if let Some(action) = self.menu_bar.render(ui) {
                self.handle_menu_action(action);
            }
        });

        // Crash recovery dialog (shows modal dialog if crash detected)
        self.crash_recovery.render_restore_dialog(ctx);

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

        // Tab bar with drag-and-drop support
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let _ = self.render_tab_bar(ui);
            });
        });

        // Render context menus
        self.render_context_menu(ctx);

        // Recently closed tabs menu (if visible)
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tab_id) = self.crash_recovery.render_recently_closed_menu(ui) {
                // User clicked to restore a tab - would need to restore from session manager
                // For now, just remove from recently closed list
                self.crash_recovery.remove_closed_tab(tab_id);
                // TODO: Actually restore the tab with its URL and history
            }
        });

        // Left side panels (Settings, History, Downloads)
        if self.settings_panel_visible {
            // Show the full settings UI with tabs in a window
            egui::Window::new("⚙ Settings")
                .default_width(850.0)
                .default_height(650.0)
                .resizable(true)
                .collapsible(false)
                .open(&mut self.settings_panel_visible)
                .show(ctx, |_ui| {
                    // The settings UI renders itself inside the window
                    self.settings_ui.show(ctx);
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
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.heading("⬇ Downloads");
                    ui.separator();

                    // Check if there are any downloads
                    if self.downloads.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label("No downloads");
                            ui.add_space(20.0);
                        });
                    } else {
                        // Render download list with scroll area
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Track which download actions to perform (avoid borrow issues)
                            let mut pause_id: Option<DownloadId> = None;
                            let mut resume_id: Option<DownloadId> = None;
                            let mut cancel_id: Option<DownloadId> = None;
                            let mut open_id: Option<DownloadId> = None;

                            for download in &self.downloads {
                                ui.group(|ui| {
                                    // Filename as header
                                    ui.strong(&download.filename);

                                    // Progress bar
                                    let progress = if download.total_bytes > 0 {
                                        download.downloaded_bytes as f32
                                            / download.total_bytes as f32
                                    } else {
                                        0.0
                                    };

                                    let progress_bar =
                                        egui::ProgressBar::new(progress).show_percentage();
                                    ui.add(progress_bar);

                                    // Size information: "4.5 MB / 10 MB"
                                    let size_info = if download.total_bytes > 0 {
                                        format!(
                                            "{} / {}",
                                            Self::format_size(download.downloaded_bytes),
                                            Self::format_size(download.total_bytes)
                                        )
                                    } else {
                                        Self::format_size(download.downloaded_bytes)
                                    };
                                    ui.label(size_info);

                                    // Speed and ETA (only for downloading status)
                                    if matches!(download.status, DownloadDisplayStatus::Downloading)
                                    {
                                        ui.horizontal(|ui| {
                                            // Download speed
                                            ui.label(format!(
                                                "Speed: {}",
                                                Self::format_speed(download.bytes_per_second)
                                            ));

                                            ui.separator();

                                            // ETA
                                            if download.eta_seconds > 0 {
                                                ui.label(format!(
                                                    "ETA: {}",
                                                    Self::format_time(download.eta_seconds)
                                                ));
                                            }
                                        });
                                    }

                                    // Status and action buttons
                                    ui.horizontal(|ui| {
                                        // Status indicator
                                        let status_text = match &download.status {
                                            DownloadDisplayStatus::Downloading => "⬇ Downloading",
                                            DownloadDisplayStatus::Paused => "⏸ Paused",
                                            DownloadDisplayStatus::Complete => "✓ Complete",
                                            DownloadDisplayStatus::Failed(msg) => {
                                                &format!("✗ Failed: {}", msg)
                                            }
                                        };
                                        ui.label(status_text);

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                // Action buttons based on status
                                                match &download.status {
                                                    DownloadDisplayStatus::Downloading => {
                                                        if ui.small_button("⏸ Pause").clicked() {
                                                            pause_id = Some(download.id);
                                                        }
                                                        if ui.small_button("✕ Cancel").clicked() {
                                                            cancel_id = Some(download.id);
                                                        }
                                                    }
                                                    DownloadDisplayStatus::Paused => {
                                                        if ui.small_button("▶ Resume").clicked() {
                                                            resume_id = Some(download.id);
                                                        }
                                                        if ui.small_button("✕ Cancel").clicked() {
                                                            cancel_id = Some(download.id);
                                                        }
                                                    }
                                                    DownloadDisplayStatus::Complete => {
                                                        if ui.small_button("📂 Open").clicked() {
                                                            open_id = Some(download.id);
                                                        }
                                                    }
                                                    DownloadDisplayStatus::Failed(_) => {
                                                        // No action buttons for failed downloads
                                                    }
                                                }
                                            },
                                        );
                                    });
                                });

                                ui.add_space(5.0);
                            }

                            // TODO: Process download actions (pause_id, resume_id, cancel_id, open_id)
                            // These would be sent via message bus to the downloads_manager
                            // For now, we just collect them but don't process
                            let _ = (pause_id, resume_id, cancel_id, open_id);
                        });
                    }

                    ui.separator();

                    // Bottom buttons
                    ui.horizontal(|ui| {
                        if ui.button("Clear Completed").clicked() {
                            self.clear_completed_downloads();
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                self.toggle_downloads_panel();
                            }
                        });
                    });
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

                    // Blocked content count badge from ad blocker
                    if self.blocked_content_count > 0 {
                        ui.separator();
                        let blocked_text = format!("🛡 {} blocked", self.blocked_content_count);
                        ui.label(egui::RichText::new(blocked_text).color(egui::Color32::from_rgb(0, 150, 0)));
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

    /// Render the tab bar with drag-and-drop support
    fn render_tab_bar(&mut self, ui: &mut egui::Ui) -> Result<(), ComponentError> {
        use tab_drag_ui::{render_drop_indicator, render_ghost_tab};

        let mut tab_to_close: Option<TabId> = None;
        let mut tab_for_context_menu: Option<TabId> = None;
        let mut tab_rects: Vec<(TabId, egui::Rect)> = Vec::new();

        // Track total tab width for overflow handling
        let mut total_tab_width = 0.0f32;
        let available_width = ui.available_width() - 40.0; // Reserve space for new tab button

        // Apply scroll offset if overflowing
        if self.tab_overflow.is_overflowing() {
            ui.add_space(-self.tab_overflow.offset());
        }

        // Render tabs in order
        for (index, &tab_id) in self.tab_order.iter().enumerate() {
            if let Some(tab) = self.tabs.get(&tab_id) {
                let is_active = index == self.active_tab_index;
                let is_being_dragged = self.tab_drag_state.dragging_tab == Some(tab_id);

                // Show loading indicator if tab is loading
                let label = if tab.loading {
                    format!("⟳ {}", tab.title)
                } else {
                    tab.title.clone()
                };

                // Dim the tab if it's being dragged
                let alpha_multiplier = if is_being_dragged { 0.3 } else { 1.0 };

                ui.scope(|ui| {
                    if is_being_dragged {
                        ui.style_mut().visuals.widgets.inactive.bg_fill =
                            ui.style().visuals.widgets.inactive.bg_fill.linear_multiply(alpha_multiplier);
                    }

                    // Tab with hover effect
                    let tab_response = ui.selectable_label(is_active, &label);
                    let tab_rect = tab_response.rect;

                    // Store rect for drop target calculation
                    tab_rects.push((tab_id, tab_rect));
                    total_tab_width += tab_rect.width();

                    // Handle drag initiation
                    if tab_response.drag_started() {
                        self.tab_drag_state.start_drag(tab_id, tab_response.interact_pointer_pos().unwrap_or_default(), index);
                    }

                    // Handle tab click (switch to tab)
                    if tab_response.clicked() && !self.tab_drag_state.is_dragging() {
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
                });
            }
        }

        // Handle ongoing drag
        if self.tab_drag_state.is_dragging() {
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                self.tab_drag_state.update_drag(pointer_pos);
                self.tab_drag_state.calculate_drop_target(&tab_rects);

                // Render drop indicator
                if let Some(drop_index) = self.tab_drag_state.drop_target_index {
                    render_drop_indicator(ui, &tab_rects, drop_index, &self.tab_drag_visuals);
                }

                // Render ghost tab
                if let (Some(tab_id), Some(drag_pos)) = (
                    self.tab_drag_state.dragging_tab,
                    self.tab_drag_state.current_drag_pos,
                ) {
                    if let Some(tab) = self.tabs.get(&tab_id) {
                        render_ghost_tab(ui, &tab.title, drag_pos, &self.tab_drag_visuals);
                    }
                }
            }

            // End drag on pointer release
            if ui.input(|i| i.pointer.any_released()) {
                if let Some((tab_id, from_index, to_index)) = self.tab_drag_state.end_drag() {
                    // Reorder tabs
                    let _ = self.reorder_tab(from_index, to_index);
                }
            }

            // Cancel drag on Escape
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.tab_drag_state.cancel_drag();
            }
        }

        // Update overflow handler
        self.tab_overflow.update(available_width, total_tab_width);

        // Handle scroll wheel for tab overflow
        ui.input(|i| {
            if self.tab_overflow.is_overflowing() {
                self.tab_overflow.handle_scroll(i.smooth_scroll_delta);
            }
        });

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

        // Show scroll indicators if overflowing
        if self.tab_overflow.is_overflowing() {
            if self.tab_overflow.offset() > 0.0 {
                ui.label("◀"); // Left scroll indicator
            }
            if self.tab_overflow.offset() < self.tab_overflow.max_scroll {
                ui.label("▶"); // Right scroll indicator
            }
        }

        Ok(())
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
