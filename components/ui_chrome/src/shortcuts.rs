// @implements: REQ-UI-006
//! Keyboard Shortcuts Handler
//!
//! Handles keyboard shortcut registration and matching.

use std::collections::HashMap;

/// Key modifier state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Actions triggered by keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    NewWindow,
    NewTab,
    CloseTab,
    CloseWindow,
    Reload,
    ReloadIgnoreCache,
    Stop,
    GoBack,
    GoForward,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleFullscreen,
    Find,
    FindNext,
    FindPrevious,
    ShowDevTools,
    ShowBookmarks,
    ShowHistory,
    ShowDownloads,
    FocusAddressBar,
    Quit,
}

/// Shortcut key combination
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ShortcutKey {
    key: String,
    modifiers: KeyModifiers,
}

/// Error types for shortcut operations
#[derive(Debug, Clone, PartialEq)]
pub enum ShortcutError {
    ShortcutNotFound,
    DuplicateShortcut,
}

impl std::fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortcutError::ShortcutNotFound => write!(f, "Shortcut not found"),
            ShortcutError::DuplicateShortcut => write!(f, "Shortcut already registered"),
        }
    }
}

impl std::error::Error for ShortcutError {}

/// Shortcut Handler state
#[derive(Debug, Clone)]
pub struct ShortcutHandler {
    shortcuts: HashMap<ShortcutKey, ShortcutAction>,
}

impl ShortcutHandler {
    /// Create a new Shortcut Handler
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
        }
    }

    /// Get the number of registered shortcuts
    pub fn shortcut_count(&self) -> usize {
        self.shortcuts.len()
    }

    /// Register a keyboard shortcut
    pub fn register(&mut self, key: String, modifiers: KeyModifiers, action: ShortcutAction) {
        let shortcut_key = ShortcutKey { key, modifiers };
        self.shortcuts.insert(shortcut_key, action);
    }

    /// Match a key press to a shortcut action
    pub fn match_shortcut(&self, key: &str, modifiers: &KeyModifiers) -> Option<ShortcutAction> {
        let shortcut_key = ShortcutKey {
            key: key.to_string(),
            modifiers: *modifiers,
        };
        self.shortcuts.get(&shortcut_key).copied()
    }

    /// Unregister a keyboard shortcut
    pub fn unregister(&mut self, key: &str, modifiers: &KeyModifiers) -> Result<(), ShortcutError> {
        let shortcut_key = ShortcutKey {
            key: key.to_string(),
            modifiers: *modifiers,
        };
        self.shortcuts
            .remove(&shortcut_key)
            .ok_or(ShortcutError::ShortcutNotFound)?;
        Ok(())
    }

    /// Get all registered shortcuts
    pub fn get_all_shortcuts(&self) -> Vec<(String, KeyModifiers, ShortcutAction)> {
        self.shortcuts
            .iter()
            .map(|(key, action)| (key.key.clone(), key.modifiers, *action))
            .collect()
    }

    /// Clear all shortcuts
    pub fn clear(&mut self) {
        self.shortcuts.clear();
    }

    /// Register default browser shortcuts
    pub fn register_defaults(&mut self) {
        // New window/tab
        self.register(
            "N".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::NewWindow,
        );
        self.register(
            "T".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::NewTab,
        );

        // Close
        self.register(
            "W".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::CloseTab,
        );

        // Navigation
        self.register(
            "R".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::Reload,
        );

        // Address bar
        self.register(
            "L".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::FocusAddressBar,
        );

        // Find
        self.register(
            "F".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::Find,
        );

        // Zoom
        self.register(
            "+".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::ZoomIn,
        );
        self.register(
            "-".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::ZoomOut,
        );
        self.register(
            "0".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
            ShortcutAction::ZoomReset,
        );

        // Dev tools
        self.register(
            "I".to_string(),
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: true,
                meta: false,
            },
            ShortcutAction::ShowDevTools,
        );
    }
}

impl Default for ShortcutHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_handler_new_creates_empty() {
        let handler = ShortcutHandler::new();
        assert_eq!(handler.shortcut_count(), 0);
    }

    #[test]
    fn shortcut_handler_register_increments_count() {
        let mut handler = ShortcutHandler::new();
        let modifiers = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        };
        handler.register("T".to_string(), modifiers, ShortcutAction::NewTab);
        assert_eq!(handler.shortcut_count(), 1);
    }

    #[test]
    fn shortcut_handler_match_returns_action() {
        let mut handler = ShortcutHandler::new();
        let modifiers = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        };
        handler.register("N".to_string(), modifiers, ShortcutAction::NewWindow);

        let action = handler.match_shortcut("N", &modifiers);
        assert_eq!(action, Some(ShortcutAction::NewWindow));
    }

    #[test]
    fn shortcut_handler_no_match_returns_none() {
        let handler = ShortcutHandler::new();
        let modifiers = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        };

        let action = handler.match_shortcut("X", &modifiers);
        assert_eq!(action, None);
    }

    #[test]
    fn shortcut_handler_unregister_removes_shortcut() {
        let mut handler = ShortcutHandler::new();
        let modifiers = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        };
        handler.register("W".to_string(), modifiers, ShortcutAction::CloseTab);
        handler.unregister("W", &modifiers).unwrap();
        assert_eq!(handler.shortcut_count(), 0);
    }

    #[test]
    fn shortcut_handler_clear_removes_all() {
        let mut handler = ShortcutHandler::new();
        handler.register_defaults();
        assert!(handler.shortcut_count() > 0);
        handler.clear();
        assert_eq!(handler.shortcut_count(), 0);
    }

    #[test]
    fn key_modifiers_equality() {
        let mod1 = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        };
        let mod2 = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        };
        assert_eq!(mod1, mod2);
    }
}
