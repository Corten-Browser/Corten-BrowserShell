// @implements: REQ-006
//! Windows-specific window implementation (Win32 API stub)

use async_trait::async_trait;
use shared_types::window::{WindowConfig, WindowId, WindowUpdate, PlatformEvent};
use shared_types::error::WindowError;
use crate::platform_window::PlatformWindow;

/// Windows window implementation (stub for now)
pub struct WindowsWindow {
    // Stub - will be filled with Win32 API implementation in future
}

impl WindowsWindow {
    /// Create a new Windows window
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PlatformWindow for WindowsWindow {
    async fn create(&mut self, _config: WindowConfig) -> Result<WindowId, WindowError> {
        unimplemented!("WindowsWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn close(&mut self, _id: WindowId) -> Result<(), WindowError> {
        unimplemented!("WindowsWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn update(&mut self, _id: WindowId, _update: WindowUpdate) -> Result<(), WindowError> {
        unimplemented!("WindowsWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn handle_event(&mut self, _id: WindowId, _event: PlatformEvent) -> Result<(), WindowError> {
        unimplemented!("WindowsWindow not yet implemented - use MockPlatformWindow for testing")
    }

    fn get_config(&self, _id: WindowId) -> Option<&WindowConfig> {
        unimplemented!("WindowsWindow not yet implemented - use MockPlatformWindow for testing")
    }
}
