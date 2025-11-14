// @implements: REQ-UI-003
//! Toolbar Widget
//!
//! Browser toolbar with navigation buttons (back, forward, reload, stop, etc.)

use std::collections::HashMap;

/// Toolbar buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolbarButton {
    Back,
    Forward,
    Reload,
    Stop,
    Home,
    Bookmarks,
}

/// Actions triggered by toolbar buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    GoBack,
    GoForward,
    Reload,
    Stop,
    GoHome,
    ShowBookmarks,
}

/// Button state
#[derive(Debug, Clone)]
struct ButtonState {
    enabled: bool,
    visible: bool,
}

/// Toolbar widget state
#[derive(Debug, Clone)]
pub struct Toolbar {
    buttons: HashMap<ToolbarButton, ButtonState>,
    is_loading: bool,
}

impl Toolbar {
    /// Create a new Toolbar widget
    pub fn new() -> Self {
        let mut buttons = HashMap::new();

        // Initialize all toolbar buttons
        buttons.insert(
            ToolbarButton::Back,
            ButtonState {
                enabled: false,
                visible: true,
            },
        );
        buttons.insert(
            ToolbarButton::Forward,
            ButtonState {
                enabled: false,
                visible: true,
            },
        );
        buttons.insert(
            ToolbarButton::Reload,
            ButtonState {
                enabled: true,
                visible: true,
            },
        );
        buttons.insert(
            ToolbarButton::Stop,
            ButtonState {
                enabled: false,
                visible: true,
            },
        );
        buttons.insert(
            ToolbarButton::Home,
            ButtonState {
                enabled: true,
                visible: true,
            },
        );
        buttons.insert(
            ToolbarButton::Bookmarks,
            ButtonState {
                enabled: true,
                visible: true,
            },
        );

        Self {
            buttons,
            is_loading: false,
        }
    }

    /// Get the number of buttons in the toolbar
    pub fn button_count(&self) -> usize {
        self.buttons.len()
    }

    /// Check if a button is enabled
    pub fn is_button_enabled(&self, button: ToolbarButton) -> bool {
        self.buttons
            .get(&button)
            .map(|state| state.enabled)
            .unwrap_or(false)
    }

    /// Set button enabled state
    pub fn set_button_enabled(&mut self, button: ToolbarButton, enabled: bool) {
        if let Some(state) = self.buttons.get_mut(&button) {
            state.enabled = enabled;
        }
    }

    /// Check if a button is visible
    pub fn is_button_visible(&self, button: ToolbarButton) -> bool {
        self.buttons
            .get(&button)
            .map(|state| state.visible)
            .unwrap_or(false)
    }

    /// Set button visible state
    pub fn set_button_visible(&mut self, button: ToolbarButton, visible: bool) {
        if let Some(state) = self.buttons.get_mut(&button) {
            state.visible = visible;
        }
    }

    /// Handle button click
    pub fn handle_click(&self, button: ToolbarButton) -> Option<ToolbarAction> {
        // Only produce action if button is enabled
        if !self.is_button_enabled(button) {
            return None;
        }

        Some(match button {
            ToolbarButton::Back => ToolbarAction::GoBack,
            ToolbarButton::Forward => ToolbarAction::GoForward,
            ToolbarButton::Reload => ToolbarAction::Reload,
            ToolbarButton::Stop => ToolbarAction::Stop,
            ToolbarButton::Home => ToolbarAction::GoHome,
            ToolbarButton::Bookmarks => ToolbarAction::ShowBookmarks,
        })
    }

    /// Update navigation state (can go back/forward/stop)
    pub fn update_navigation_state(&mut self, can_go_back: bool, can_go_forward: bool, is_loading: bool) {
        self.set_button_enabled(ToolbarButton::Back, can_go_back);
        self.set_button_enabled(ToolbarButton::Forward, can_go_forward);
        self.set_loading(is_loading);
    }

    /// Set loading state (shows stop button, hides reload)
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;

        if loading {
            self.set_button_enabled(ToolbarButton::Stop, true);
            self.set_button_visible(ToolbarButton::Reload, false);
            self.set_button_visible(ToolbarButton::Stop, true);
        } else {
            self.set_button_enabled(ToolbarButton::Stop, false);
            self.set_button_visible(ToolbarButton::Reload, true);
            self.set_button_visible(ToolbarButton::Stop, false);
        }
    }

    /// Get button tooltip text
    pub fn get_button_tooltip(&self, button: ToolbarButton) -> &'static str {
        match button {
            ToolbarButton::Back => "Go back",
            ToolbarButton::Forward => "Go forward",
            ToolbarButton::Reload => "Reload page",
            ToolbarButton::Stop => "Stop loading",
            ToolbarButton::Home => "Go to home page",
            ToolbarButton::Bookmarks => "Show bookmarks",
        }
    }

    /// Get all toolbar buttons
    pub fn get_all_buttons(&self) -> Vec<ToolbarButton> {
        self.buttons.keys().copied().collect()
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_new_creates_with_buttons() {
        let toolbar = Toolbar::new();
        assert_eq!(toolbar.button_count(), 6);
    }

    #[test]
    fn toolbar_default_button_states() {
        let toolbar = Toolbar::new();
        assert!(!toolbar.is_button_enabled(ToolbarButton::Back));
        assert!(!toolbar.is_button_enabled(ToolbarButton::Forward));
        assert!(toolbar.is_button_enabled(ToolbarButton::Reload));
        assert!(!toolbar.is_button_enabled(ToolbarButton::Stop));
        assert!(toolbar.is_button_enabled(ToolbarButton::Home));
        assert!(toolbar.is_button_enabled(ToolbarButton::Bookmarks));
    }

    #[test]
    fn toolbar_button_enabled_toggle() {
        let mut toolbar = Toolbar::new();
        toolbar.set_button_enabled(ToolbarButton::Back, true);
        assert!(toolbar.is_button_enabled(ToolbarButton::Back));
        toolbar.set_button_enabled(ToolbarButton::Back, false);
        assert!(!toolbar.is_button_enabled(ToolbarButton::Back));
    }

    #[test]
    fn toolbar_loading_state_changes_buttons() {
        let mut toolbar = Toolbar::new();
        toolbar.set_loading(true);
        assert!(toolbar.is_button_enabled(ToolbarButton::Stop));
        assert!(!toolbar.is_button_visible(ToolbarButton::Reload));
    }

    #[test]
    fn toolbar_action_mapping() {
        let mut toolbar = Toolbar::new();
        toolbar.set_button_enabled(ToolbarButton::Back, true);
        let action = toolbar.handle_click(ToolbarButton::Back);
        assert_eq!(action, Some(ToolbarAction::GoBack));
    }
}
