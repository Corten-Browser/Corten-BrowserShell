//! Window configuration types
//!
//! This module provides configuration structures for browser windows.

use serde::{Deserialize, Serialize};

/// Configuration for creating or updating a browser window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    pub title: String,

    /// Window width in pixels
    pub width: u32,

    /// Window height in pixels
    pub height: u32,

    /// Window X position (None for default/auto)
    pub x: Option<i32>,

    /// Window Y position (None for default/auto)
    pub y: Option<i32>,

    /// Whether the window should be fullscreen
    pub fullscreen: bool,

    /// Whether the window is resizable
    pub resizable: bool,

    /// Whether the window has decorations (title bar, borders)
    pub decorations: bool,

    /// Whether the window should always be on top
    pub always_on_top: bool,

    /// Whether the window should be excluded from the taskbar
    pub skip_taskbar: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "CortenBrowser".to_string(),
            width: 1024,
            height: 768,
            x: None,
            y: None,
            fullscreen: false,
            resizable: true,
            decorations: true,
            always_on_top: false,
            skip_taskbar: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_window_config() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "CortenBrowser");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
    }

    #[test]
    fn test_custom_window_config() {
        let config = WindowConfig {
            title: "Custom".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        };
        assert_eq!(config.title, "Custom");
        assert_eq!(config.width, 800);
    }
}
