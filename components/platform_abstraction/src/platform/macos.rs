// @implements: REQ-006
//! macOS-specific window implementation (Cocoa stub)

use async_trait::async_trait;
use shared_types::window::{WindowConfig, WindowId, WindowUpdate, PlatformEvent};
use shared_types::error::WindowError;
use crate::platform_window::PlatformWindow;

/// macOS window implementation (stub for now)
pub struct MacWindow {
    // Stub - will be filled with Cocoa implementation in future
}

impl MacWindow {
    /// Create a new macOS window
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PlatformWindow for MacWindow {
    async fn create(&mut self, _config: WindowConfig) -> Result<WindowId, WindowError> {
        unimplemented!("MacWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn close(&mut self, _id: WindowId) -> Result<(), WindowError> {
        unimplemented!("MacWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn update(&mut self, _id: WindowId, _update: WindowUpdate) -> Result<(), WindowError> {
        unimplemented!("MacWindow not yet implemented - use MockPlatformWindow for testing")
    }

    async fn handle_event(&mut self, _id: WindowId, _event: PlatformEvent) -> Result<(), WindowError> {
        unimplemented!("MacWindow not yet implemented - use MockPlatformWindow for testing")
    }

    fn get_config(&self, _id: WindowId) -> Option<&WindowConfig> {
        unimplemented!("MacWindow not yet implemented - use MockPlatformWindow for testing")
    }
}
