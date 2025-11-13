// @implements: REQ-001, REQ-002
//! Platform window creation trait and mock implementation
//!
//! This module defines the cross-platform PlatformWindow trait and a mock
//! implementation for testing.

use async_trait::async_trait;
use shared_types::window::{WindowConfig, WindowId, WindowUpdate, PlatformEvent};
use shared_types::error::WindowError;

/// Platform window creation trait
#[async_trait]
pub trait PlatformWindow: Send + Sync {
    /// Create a new platform window
    async fn create(&mut self, config: WindowConfig) -> Result<WindowId, WindowError>;

    /// Close a platform window
    async fn close(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Update window properties
    async fn update(
        &mut self,
        id: WindowId,
        update: WindowUpdate,
    ) -> Result<(), WindowError>;

    /// Handle platform events
    async fn handle_event(
        &mut self,
        id: WindowId,
        event: PlatformEvent,
    ) -> Result<(), WindowError>;

    /// Get window configuration
    fn get_config(&self, id: WindowId) -> Option<&WindowConfig>;
}

/// Mock platform window for testing
pub struct MockPlatformWindow {
    // Stub implementation - will be filled in GREEN phase
}

impl MockPlatformWindow {
    /// Create a new mock platform window
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PlatformWindow for MockPlatformWindow {
    async fn create(&mut self, _config: WindowConfig) -> Result<WindowId, WindowError> {
        // Stub - will implement in GREEN phase
        unimplemented!("MockPlatformWindow::create not yet implemented")
    }

    async fn close(&mut self, _id: WindowId) -> Result<(), WindowError> {
        // Stub - will implement in GREEN phase
        unimplemented!("MockPlatformWindow::close not yet implemented")
    }

    async fn update(
        &mut self,
        _id: WindowId,
        _update: WindowUpdate,
    ) -> Result<(), WindowError> {
        // Stub - will implement in GREEN phase
        unimplemented!("MockPlatformWindow::update not yet implemented")
    }

    async fn handle_event(
        &mut self,
        _id: WindowId,
        _event: PlatformEvent,
    ) -> Result<(), WindowError> {
        // Stub - will implement in GREEN phase
        unimplemented!("MockPlatformWindow::handle_event not yet implemented")
    }

    fn get_config(&self, _id: WindowId) -> Option<&WindowConfig> {
        // Stub - will implement in GREEN phase
        unimplemented!("MockPlatformWindow::get_config not yet implemented")
    }
}

impl Default for MockPlatformWindow {
    fn default() -> Self {
        Self::new()
    }
}
