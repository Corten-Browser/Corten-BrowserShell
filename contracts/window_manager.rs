// Window Manager Contract
// Version: 0.17.0
//
// This contract defines the interface for window lifecycle management.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Window identifier (UUID-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u128);

impl WindowId {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(timestamp)
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

/// Window error types
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Window not found: {0:?}")]
    NotFound(WindowId),

    #[error("Window creation failed: {0}")]
    CreationFailed(String),

    #[error("Invalid window configuration: {0}")]
    InvalidConfig(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Window representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: WindowId,
    pub config: WindowConfig,
    pub tabs: Vec<super::tab_manager::TabId>,
    pub active_tab: Option<super::tab_manager::TabId>,
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
