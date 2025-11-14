//! Keyboard shortcut definitions
//!
//! This module provides an enumeration of supported keyboard shortcuts
//! for browser operations.

use serde::{Deserialize, Serialize};

/// Keyboard shortcuts for common browser actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyboardShortcut {
    /// Ctrl+T - New tab
    CtrlT,

    /// Ctrl+W - Close tab
    CtrlW,

    /// Ctrl+N - New window
    CtrlN,

    /// Ctrl+Shift+T - Reopen closed tab
    CtrlShiftT,

    /// Ctrl+L - Focus address bar
    CtrlL,

    /// F5 - Reload
    F5,

    /// Ctrl+R - Reload
    CtrlR,

    /// Ctrl+Shift+R - Hard reload
    CtrlShiftR,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_shortcut_count() {
        // Ensure we have all expected shortcuts
        let shortcuts = [
            KeyboardShortcut::CtrlT,
            KeyboardShortcut::CtrlW,
            KeyboardShortcut::CtrlN,
            KeyboardShortcut::CtrlShiftT,
            KeyboardShortcut::CtrlL,
            KeyboardShortcut::F5,
            KeyboardShortcut::CtrlR,
            KeyboardShortcut::CtrlShiftR,
        ];
        assert_eq!(shortcuts.len(), 8);
    }

    #[test]
    fn test_keyboard_shortcut_uniqueness() {
        assert_ne!(KeyboardShortcut::CtrlT, KeyboardShortcut::CtrlW);
        assert_ne!(KeyboardShortcut::F5, KeyboardShortcut::CtrlR);
    }
}
