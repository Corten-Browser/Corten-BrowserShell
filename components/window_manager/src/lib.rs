//! Window Manager Component
//!
//! Browser window lifecycle management including creation, resizing, focus, and multi-window support.
//!
//! # Multi-Window Tab Drag
//!
//! The [`tab_drag`] module provides cross-window tab drag and drop functionality:
//!
//! ```rust,ignore
//! use window_manager::tab_drag::{TabDragManager, TabTransferData};
//!
//! let mut manager = TabDragManager::new();
//! manager.start_drag(tab_id, window_id, position, transfer_data);
//! ```
//!
//! # Picture-in-Picture
//!
//! The [`pip`] module provides floating video window support:
//!
//! ```rust,ignore
//! use window_manager::pip::{PipManager, PipConfig, PipVideoSource};
//!
//! let mut manager = PipManager::new();
//! let source = PipVideoSource::new(tab_id, "video-element-1".to_string());
//! let pip_id = manager.create_pip_window(source, PipConfig::default())?;
//! ```

pub mod pip;
pub mod tab_drag;

use platform_abstraction::{PlatformHandle, PlatformWindow};
use shared_types::{TabId, WindowConfig, WindowError, WindowId};
use std::collections::HashMap;

// Re-export commonly used tab drag types
pub use tab_drag::{
    CrossWindowMessage, DragFeedback, DragSession, DropIndicator, HistoryEntry, Position,
    Rectangle, TabDragError, TabDragManager, TabDragState, TabTransferData, WindowDropTarget,
};

// Re-export commonly used PiP types
pub use pip::{
    ExtractedVideoInfo, PipBounds, PipConfig, PipControlAction, PipControls, PipCorner, PipError,
    PipManager, PipPosition, PipSize, PipState, PipVideoSource, PipWindow, PipWindowId,
    StubVideoExtractor, VideoExtractionError, VideoExtractor, DEFAULT_PIP_HEIGHT,
    DEFAULT_PIP_OPACITY, DEFAULT_PIP_WIDTH, MAX_PIP_HEIGHT, MAX_PIP_WINDOWS, MAX_PIP_WIDTH,
    MIN_PIP_HEIGHT, MIN_PIP_WIDTH,
};

/// Represents a browser window
pub struct Window<P: PlatformWindow> {
    /// Unique identifier for this window
    pub id: WindowId,
    /// Window configuration
    pub config: WindowConfig,
    /// List of tabs in this window
    pub tabs: Vec<TabId>,
    /// Currently active tab
    pub active_tab: Option<TabId>,
    /// Platform-specific window handle
    pub platform_handle: PlatformHandle,
    /// Platform window implementation
    platform_window: P,
}

impl<P: PlatformWindow> Window<P> {
    /// Create a new window with the given configuration and platform window
    pub fn new(config: WindowConfig, platform_window: P) -> Self {
        let id = WindowId::new();
        let platform_handle = platform_window.get_handle();

        Self {
            id,
            config,
            tabs: Vec::new(),
            active_tab: None,
            platform_handle,
            platform_window,
        }
    }

    /// Add a tab to this window
    pub fn add_tab(&mut self, tab_id: TabId) {
        self.tabs.push(tab_id);
    }

    /// Remove a tab from this window
    pub fn remove_tab(&mut self, tab_id: &TabId) {
        self.tabs.retain(|id| id != tab_id);
        if self.active_tab == Some(*tab_id) {
            self.active_tab = None;
        }
    }

    /// Set the active tab
    pub fn set_active_tab(&mut self, tab_id: Option<TabId>) {
        self.active_tab = tab_id;
    }

    /// Resize the window
    pub async fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        self.platform_window.resize(width, height)?;
        self.config.width = width;
        self.config.height = height;
        Ok(())
    }

    /// Focus the window
    pub async fn focus(&mut self) -> Result<(), WindowError> {
        self.platform_window.focus()
    }

    /// Destroy the platform window
    pub async fn destroy(&mut self) -> Result<(), WindowError> {
        self.platform_window.destroy()
    }
}

/// Manages browser windows
pub struct WindowManager<P: PlatformWindow> {
    /// Map of window IDs to windows
    windows: HashMap<WindowId, Window<P>>,
}

impl<P: PlatformWindow> WindowManager<P> {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    /// Create a new browser window
    ///
    /// # Arguments
    ///
    /// * `config` - Window configuration
    ///
    /// # Returns
    ///
    /// * `Ok(WindowId)` - The ID of the newly created window
    /// * `Err(WindowError)` - If window creation fails
    pub async fn create_window(&mut self, config: WindowConfig) -> Result<WindowId, WindowError> {
        let platform_window = P::create(&config)?;
        let window = Window::new(config, platform_window);
        let window_id = window.id;

        self.windows.insert(window_id, window);
        Ok(window_id)
    }

    /// Close a browser window
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the window to close
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully closed
    /// * `Err(WindowError)` - If window not found or close fails
    pub async fn close_window(&mut self, id: WindowId) -> Result<(), WindowError> {
        let mut window = self.windows.remove(&id).ok_or(WindowError::NotFound(id))?;

        window.destroy().await?;
        Ok(())
    }

    /// Get all active window IDs
    ///
    /// # Returns
    ///
    /// Vector of all active window IDs
    pub fn get_windows(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }

    /// Get window configuration
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the window
    ///
    /// # Returns
    ///
    /// * `Some(WindowConfig)` - The window's configuration
    /// * `None` - If window not found
    pub fn get_window_config(&self, id: WindowId) -> Option<WindowConfig> {
        self.windows.get(&id).map(|w| w.config.clone())
    }

    /// Resize window to new dimensions
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the window to resize
    /// * `width` - New width in pixels
    /// * `height` - New height in pixels
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully resized
    /// * `Err(WindowError)` - If window not found or resize fails
    pub async fn resize_window(
        &mut self,
        id: WindowId,
        width: u32,
        height: u32,
    ) -> Result<(), WindowError> {
        let window = self.windows.get_mut(&id).ok_or(WindowError::NotFound(id))?;
        window.resize(width, height).await
    }

    /// Bring window to front and focus
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the window to focus
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully focused
    /// * `Err(WindowError)` - If window not found or focus fails
    pub async fn focus_window(&mut self, id: WindowId) -> Result<(), WindowError> {
        let window = self.windows.get_mut(&id).ok_or(WindowError::NotFound(id))?;
        window.focus().await
    }
}

impl<P: PlatformWindow> Default for WindowManager<P> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock platform window for internal tests
    struct MockPlatformWindow {
        handle: PlatformHandle,
    }

    impl PlatformWindow for MockPlatformWindow {
        fn create(_config: &WindowConfig) -> Result<Self, WindowError> {
            Ok(Self {
                handle: PlatformHandle::Stub(platform_abstraction::StubHandle { id: 1 }),
            })
        }

        fn destroy(&mut self) -> Result<(), WindowError> {
            Ok(())
        }

        fn show(&mut self) -> Result<(), WindowError> {
            Ok(())
        }

        fn hide(&mut self) -> Result<(), WindowError> {
            Ok(())
        }

        fn resize(&mut self, _width: u32, _height: u32) -> Result<(), WindowError> {
            Ok(())
        }

        fn move_to(&mut self, _x: i32, _y: i32) -> Result<(), WindowError> {
            Ok(())
        }

        fn focus(&mut self) -> Result<(), WindowError> {
            Ok(())
        }

        fn get_handle(&self) -> PlatformHandle {
            self.handle.clone()
        }
    }

    #[tokio::test]
    async fn test_window_manager_new() {
        let manager = WindowManager::<MockPlatformWindow>::new();
        assert_eq!(manager.get_windows().len(), 0);
    }
}
