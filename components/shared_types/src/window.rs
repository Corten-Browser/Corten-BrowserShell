// @implements: REQ-004
//! Window management types
//!
//! This module provides types for window lifecycle management.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::WindowError;
use crate::tab::TabId;

/// Window identifier (UUID-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u128);

impl WindowId {
    /// Create a new unique WindowId
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(timestamp)
    }
}

impl Default for WindowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub fullscreen: bool,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
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

/// Window update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowUpdate {
    Resize { width: u32, height: u32 },
    Move { x: i32, y: i32 },
    SetTitle { title: String },
    SetFullscreen { fullscreen: bool },
    Focus,
    Minimize,
    Maximize,
    Restore,
}

/// Platform window event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformEvent {
    Resized { width: u32, height: u32 },
    Moved { x: i32, y: i32 },
    CloseRequested,
    Focused,
    Unfocused,
    KeyboardInput { key: String, pressed: bool },
    MouseInput { x: i32, y: i32, button: String, pressed: bool },
}

/// Window representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: WindowId,
    pub config: WindowConfig,
    pub tabs: Vec<TabId>,
    pub active_tab: Option<TabId>,
    pub is_focused: bool,
    pub is_fullscreen: bool,
}

/// Window Manager interface
#[async_trait]
pub trait WindowManager: Send + Sync {
    /// Create a new browser window
    async fn create_window(&mut self, config: WindowConfig) -> Result<WindowId, WindowError>;

    /// Close a window
    async fn close_window(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Get all windows
    fn get_windows(&self) -> Vec<&Window>;

    /// Get window by ID
    fn get_window(&self, id: WindowId) -> Option<&Window>;

    /// Update window properties
    async fn update_window(
        &mut self,
        id: WindowId,
        update: WindowUpdate,
    ) -> Result<(), WindowError>;

    /// Handle platform window events
    async fn handle_platform_event(
        &mut self,
        window_id: WindowId,
        event: PlatformEvent,
    ) -> Result<(), WindowError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_id_creation_is_unique() {
        let id1 = WindowId::new();
        std::thread::sleep(std::time::Duration::from_nanos(10));
        let id2 = WindowId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn window_id_default_creates_new() {
        let id = WindowId::default();
        assert!(id.0 > 0);
    }

    #[test]
    fn window_config_default_has_reasonable_values() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "CortenBrowser");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.fullscreen, false);
        assert_eq!(config.resizable, true);
        assert_eq!(config.decorations, true);
    }

    #[test]
    fn window_update_can_be_created() {
        let update = WindowUpdate::Resize { width: 800, height: 600 };
        match update {
            WindowUpdate::Resize { width, height } => {
                assert_eq!(width, 800);
                assert_eq!(height, 600);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn platform_event_can_be_created() {
        let event = PlatformEvent::Resized { width: 1920, height: 1080 };
        match event {
            PlatformEvent::Resized { width, height } => {
                assert_eq!(width, 1920);
                assert_eq!(height, 1080);
            }
            _ => panic!("Wrong variant"),
        }
    }
}
