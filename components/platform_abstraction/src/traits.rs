//! Platform window trait definition
//!
//! This module defines the `PlatformWindow` trait that all platform-specific
//! window implementations must implement.

use crate::PlatformHandle;
use shared_types::{WindowConfig, WindowError};

/// Platform-specific window implementation trait
///
/// This trait defines the interface that all platform-specific window
/// implementations must satisfy. It provides methods for window lifecycle
/// management (create, destroy), visibility control (show, hide), and
/// manipulation (resize, move, focus).
pub trait PlatformWindow: Sized {
    /// Create a new platform window with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Window configuration specifying size, position, and attributes
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created window
    /// * `Err(WindowError)` - Window creation failed
    fn create(config: &WindowConfig) -> Result<Self, WindowError>;

    /// Destroy the window and release all resources
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully destroyed
    /// * `Err(WindowError)` - Failed to destroy window
    fn destroy(&mut self) -> Result<(), WindowError>;

    /// Show the window (make it visible)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully shown
    /// * `Err(WindowError)` - Failed to show window
    fn show(&mut self) -> Result<(), WindowError>;

    /// Hide the window (make it invisible)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully hidden
    /// * `Err(WindowError)` - Failed to hide window
    fn hide(&mut self) -> Result<(), WindowError>;

    /// Resize the window to the specified dimensions
    ///
    /// # Arguments
    ///
    /// * `width` - New width in pixels
    /// * `height` - New height in pixels
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully resized
    /// * `Err(WindowError)` - Failed to resize window
    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError>;

    /// Move the window to the specified position
    ///
    /// # Arguments
    ///
    /// * `x` - New X coordinate
    /// * `y` - New Y coordinate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully moved
    /// * `Err(WindowError)` - Failed to move window
    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError>;

    /// Focus the window (bring to front and activate)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Window successfully focused
    /// * `Err(WindowError)` - Failed to focus window
    fn focus(&mut self) -> Result<(), WindowError>;

    /// Get the platform-specific window handle
    ///
    /// # Returns
    ///
    /// Platform-specific handle that can be used for native interop
    fn get_handle(&self) -> PlatformHandle;
}
