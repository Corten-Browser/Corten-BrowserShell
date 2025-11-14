// @implements: REQ-004, REQ-005
//! Window state management module

use shared_types::window::{Window, WindowConfig, WindowId, WindowUpdate};
use shared_types::tab::TabId;

/// Window state tracker
///
/// Maintains the current state of a window including configuration,
/// tabs, focus state, and fullscreen mode.
pub struct WindowState {
    /// Immutable window data
    window: Window,
}

impl WindowState {
    /// Create a new window state
    ///
    /// # Arguments
    ///
    /// * `id` - Window identifier
    /// * `config` - Initial window configuration
    ///
    /// # Returns
    ///
    /// New window state instance
    pub fn new(id: WindowId, config: WindowConfig) -> Self {
        let is_fullscreen = config.fullscreen;
        Self {
            window: Window {
                id,
                config,
                tabs: Vec::new(),
                active_tab: None,
                is_focused: false,
                is_fullscreen,
            },
        }
    }

    /// Get immutable reference to window
    ///
    /// # Returns
    ///
    /// Immutable reference to window data
    pub fn as_window(&self) -> &Window {
        &self.window
    }

    /// Apply a window update operation
    ///
    /// # Arguments
    ///
    /// * `update` - Update operation to apply
    pub fn apply_update(&mut self, update: WindowUpdate) {
        match update {
            WindowUpdate::Resize { width, height } => {
                self.window.config.width = width;
                self.window.config.height = height;
            }
            WindowUpdate::Move { x, y } => {
                self.window.config.x = Some(x);
                self.window.config.y = Some(y);
            }
            WindowUpdate::SetTitle { title } => {
                self.window.config.title = title;
            }
            WindowUpdate::SetFullscreen { fullscreen } => {
                self.window.config.fullscreen = fullscreen;
                self.window.is_fullscreen = fullscreen;
            }
            WindowUpdate::Focus => {
                self.window.is_focused = true;
            }
            WindowUpdate::Minimize => {
                // State change handled by platform
                self.window.is_focused = false;
            }
            WindowUpdate::Maximize => {
                // State change handled by platform
                self.window.is_focused = true;
            }
            WindowUpdate::Restore => {
                // State change handled by platform
                self.window.is_focused = true;
            }
        }
    }

    /// Set focus state
    ///
    /// # Arguments
    ///
    /// * `focused` - New focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.window.is_focused = focused;
    }

    /// Update window size from platform event
    ///
    /// # Arguments
    ///
    /// * `width` - New width
    /// * `height` - New height
    pub fn update_size(&mut self, width: u32, height: u32) {
        self.window.config.width = width;
        self.window.config.height = height;
    }

    /// Update window position from platform event
    ///
    /// # Arguments
    ///
    /// * `x` - New x coordinate
    /// * `y` - New y coordinate
    pub fn update_position(&mut self, x: i32, y: i32) {
        self.window.config.x = Some(x);
        self.window.config.y = Some(y);
    }

    /// Add a tab to the window
    ///
    /// # Arguments
    ///
    /// * `tab_id` - Tab identifier
    pub fn add_tab(&mut self, tab_id: TabId) {
        self.window.tabs.push(tab_id);
        if self.window.active_tab.is_none() {
            self.window.active_tab = Some(tab_id);
        }
    }

    /// Remove a tab from the window
    ///
    /// # Arguments
    ///
    /// * `tab_id` - Tab identifier
    pub fn remove_tab(&mut self, tab_id: TabId) {
        self.window.tabs.retain(|&id| id != tab_id);
        if self.window.active_tab == Some(tab_id) {
            self.window.active_tab = self.window.tabs.first().copied();
        }
    }

    /// Set active tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - Tab identifier
    pub fn set_active_tab(&mut self, tab_id: TabId) {
        if self.window.tabs.contains(&tab_id) {
            self.window.active_tab = Some(tab_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_state_new_creates_with_config() {
        let id = WindowId::new();
        let mut config = WindowConfig::default();
        config.title = "Test Window".to_string();

        let state = WindowState::new(id, config.clone());

        assert_eq!(state.window.id, id);
        assert_eq!(state.window.config.title, "Test Window");
        assert_eq!(state.window.tabs.len(), 0);
        assert!(state.window.active_tab.is_none());
        assert!(!state.window.is_focused);
    }

    #[test]
    fn apply_update_resize_changes_dimensions() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        state.apply_update(WindowUpdate::Resize { width: 1920, height: 1080 });

        assert_eq!(state.window.config.width, 1920);
        assert_eq!(state.window.config.height, 1080);
    }

    #[test]
    fn apply_update_move_changes_position() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        state.apply_update(WindowUpdate::Move { x: 100, y: 200 });

        assert_eq!(state.window.config.x, Some(100));
        assert_eq!(state.window.config.y, Some(200));
    }

    #[test]
    fn apply_update_set_title_changes_title() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        state.apply_update(WindowUpdate::SetTitle { title: "New Title".to_string() });

        assert_eq!(state.window.config.title, "New Title");
    }

    #[test]
    fn apply_update_set_fullscreen_changes_fullscreen_state() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        assert!(!state.window.is_fullscreen);

        state.apply_update(WindowUpdate::SetFullscreen { fullscreen: true });

        assert!(state.window.is_fullscreen);
        assert!(state.window.config.fullscreen);
    }

    #[test]
    fn apply_update_focus_sets_focused() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        assert!(!state.window.is_focused);

        state.apply_update(WindowUpdate::Focus);

        assert!(state.window.is_focused);
    }

    #[test]
    fn set_focused_updates_focus_state() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        state.set_focused(true);
        assert!(state.window.is_focused);

        state.set_focused(false);
        assert!(!state.window.is_focused);
    }

    #[test]
    fn update_size_changes_dimensions() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        state.update_size(1600, 900);

        assert_eq!(state.window.config.width, 1600);
        assert_eq!(state.window.config.height, 900);
    }

    #[test]
    fn update_position_changes_coordinates() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        state.update_position(50, 75);

        assert_eq!(state.window.config.x, Some(50));
        assert_eq!(state.window.config.y, Some(75));
    }

    #[test]
    fn add_tab_appends_to_tabs_list() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let tab_id = TabId::new();
        state.add_tab(tab_id);

        assert_eq!(state.window.tabs.len(), 1);
        assert_eq!(state.window.tabs[0], tab_id);
        assert_eq!(state.window.active_tab, Some(tab_id));
    }

    #[test]
    fn remove_tab_removes_from_list() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let tab_id1 = TabId::new();
        let tab_id2 = TabId::new();
        state.add_tab(tab_id1);
        state.add_tab(tab_id2);

        state.remove_tab(tab_id1);

        assert_eq!(state.window.tabs.len(), 1);
        assert_eq!(state.window.tabs[0], tab_id2);
    }

    #[test]
    fn set_active_tab_updates_active_tab() {
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let tab_id1 = TabId::new();
        let tab_id2 = TabId::new();
        state.add_tab(tab_id1);
        state.add_tab(tab_id2);

        state.set_active_tab(tab_id2);

        assert_eq!(state.window.active_tab, Some(tab_id2));
    }
}
