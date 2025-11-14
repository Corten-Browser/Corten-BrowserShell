// @implements: REQ-006
//! Browser State Management
//!
//! Manages global browser state.

use shared_types::WindowId;

/// BrowserState tracks global browser state
pub struct BrowserState {
    focused_window: Option<WindowId>,
    fullscreen: bool,
}

impl BrowserState {
    /// Create a new BrowserState
    pub fn new() -> Self {
        Self {
            focused_window: None,
            fullscreen: false,
        }
    }

    /// Get the currently focused window
    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Set the focused window
    pub fn set_focused_window(&mut self, window_id: Option<WindowId>) {
        self.focused_window = window_id;
    }

    /// Check if any window is in fullscreen mode
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Set fullscreen state
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self::new()
    }
}
