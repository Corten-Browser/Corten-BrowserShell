// @implements: REQ-006
//! Linux-specific window implementation (X11/Wayland stub)

use async_trait::async_trait;
use shared_types::window::{WindowConfig, WindowId, WindowUpdate, PlatformEvent};
use shared_types::error::WindowError;
use crate::platform_window::PlatformWindow;

/// Linux window implementation (stub for now)
pub struct LinuxWindow {
    // Stub - will be filled with X11rb or Wayland implementation in future
}

impl LinuxWindow {
    /// Create a new Linux window
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PlatformWindow for LinuxWindow {
    async fn create(&mut self, _config: WindowConfig) -> Result<WindowId, WindowError> {
        unimplemented!("LinuxWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn close(&mut self, _id: WindowId) -> Result<(), WindowError> {
        unimplemented!("LinuxWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn update(&mut self, _id: WindowId, _update: WindowUpdate) -> Result<(), WindowError> {
        unimplemented!("LinuxWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn handle_event(&mut self, _id: WindowId, _event: PlatformEvent) -> Result<(), WindowError> {
        unimplemented!("LinuxWindow not yet implemented - use MockPlatformWindow for testing")
    }

    fn get_config(&self, _id: WindowId) -> Option<&WindowConfig> {
        unimplemented!("LinuxWindow not yet implemented - use MockPlatformWindow for testing")
    }
}
