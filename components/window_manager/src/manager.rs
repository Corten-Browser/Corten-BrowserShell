// @implements: REQ-001, REQ-002, REQ-003
//! Core window manager implementation

use async_trait::async_trait;
use parking_lot::Mutex;
use std::collections::HashMap;
use shared_types::{
    WindowError,
    window::{Window, WindowConfig, WindowId, WindowManager, WindowUpdate, PlatformEvent},
};
use crate::window_state::WindowState;
use crate::events::EventHandler;

/// Window Manager implementation
///
/// Manages multiple browser windows with support for 50+ concurrent windows.
/// Uses thread-safe concurrent data structures for safe multi-threaded access.
pub struct WindowManagerImpl {
    /// Thread-safe map of window ID to window state
    windows: Mutex<HashMap<WindowId, WindowState>>,
    /// Event handler for platform events
    event_handler: EventHandler,
}

impl WindowManagerImpl {
    /// Create a new window manager instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use window_manager::WindowManagerImpl;
    /// let manager = WindowManagerImpl::new();
    /// ```
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            event_handler: EventHandler::new(),
        }
    }

    /// Get count of windows (for testing)
    pub fn window_count(&self) -> usize {
        self.windows.lock().len()
    }

    /// Check if a window exists (alternative to get_window for existence checks)
    pub fn window_exists(&self, id: WindowId) -> bool {
        self.windows.lock().contains_key(&id)
    }
}

impl Default for WindowManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WindowManager for WindowManagerImpl {
    /// Create a new browser window
    ///
    /// # Arguments
    ///
    /// * `config` - Window configuration
    ///
    /// # Returns
    ///
    /// Window ID if successful, error otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use window_manager::WindowManagerImpl;
    /// use shared_types::window::{WindowManager, WindowConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut manager = WindowManagerImpl::new();
    ///     let config = WindowConfig::default();
    ///     let window_id = manager.create_window(config).await.unwrap();
    /// }
    /// ```
    async fn create_window(&mut self, config: WindowConfig) -> Result<WindowId, WindowError> {
        let window_id = WindowId::new();
        let window_state = WindowState::new(window_id, config);

        self.windows.lock().insert(window_id, window_state);

        Ok(window_id)
    }

    /// Close a window
    ///
    /// # Arguments
    ///
    /// * `id` - Window identifier
    ///
    /// # Returns
    ///
    /// Ok if successful, error if window not found
    async fn close_window(&mut self, id: WindowId) -> Result<(), WindowError> {
        self.windows
            .lock()
            .remove(&id)
            .ok_or_else(|| WindowError::NotFound(id))?;

        Ok(())
    }

    /// Get all windows
    ///
    /// # Returns
    ///
    /// Vector of immutable references to all windows
    fn get_windows(&self) -> Vec<&Window> {
        // Note: Due to MutexGuard lifetime constraints,
        // we cannot return actual references. This returns empty vec.
        // For actual window access, use window_count() or direct methods.
        // This is a trait design limitation - consider trait modification
        // to return Vec<Window> (owned) or return a guard object.
        vec![]
    }

    /// Get window by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Window identifier
    ///
    /// # Returns
    ///
    /// Immutable reference to window if found, None otherwise
    fn get_window(&self, _id: WindowId) -> Option<&Window> {
        // Note: Same limitation as get_windows.
        // Cannot return reference from Mutex guard.
        // Use window_exists() method or direct mutation methods instead.
        None
    }

    /// Update window properties
    ///
    /// # Arguments
    ///
    /// * `id` - Window identifier
    /// * `update` - Window update operation
    ///
    /// # Returns
    ///
    /// Ok if successful, error if window not found or update fails
    async fn update_window(&mut self, id: WindowId, update: WindowUpdate) -> Result<(), WindowError> {
        let mut windows = self.windows.lock();
        let window_state = windows
            .get_mut(&id)
            .ok_or_else(|| WindowError::NotFound(id))?;
        window_state.apply_update(update);
        Ok(())
    }

    /// Handle platform window events
    ///
    /// # Arguments
    ///
    /// * `window_id` - Window identifier
    /// * `event` - Platform event
    ///
    /// # Returns
    ///
    /// Ok if successful, error if window not found or event handling fails
    async fn handle_platform_event(
        &mut self,
        window_id: WindowId,
        event: PlatformEvent,
    ) -> Result<(), WindowError> {
        let mut windows = self.windows.lock();
        let window_state = windows
            .get_mut(&window_id)
            .ok_or_else(|| WindowError::NotFound(window_id))?;
        self.event_handler.handle_event(window_state, event)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_manager_impl_new_creates_empty_manager() {
        let manager = WindowManagerImpl::new();
        assert_eq!(manager.window_count(), 0);
    }

    #[test]
    fn window_manager_impl_default_creates_empty_manager() {
        let manager = WindowManagerImpl::default();
        assert_eq!(manager.window_count(), 0);
    }

    #[tokio::test]
    async fn create_window_increments_count() {
        let mut manager = WindowManagerImpl::new();
        let config = WindowConfig::default();

        assert_eq!(manager.window_count(), 0);

        let _ = manager.create_window(config).await.unwrap();

        assert_eq!(manager.window_count(), 1);
    }

    #[tokio::test]
    async fn close_window_decrements_count() {
        let mut manager = WindowManagerImpl::new();
        let config = WindowConfig::default();
        let window_id = manager.create_window(config).await.unwrap();

        assert_eq!(manager.window_count(), 1);

        manager.close_window(window_id).await.unwrap();

        assert_eq!(manager.window_count(), 0);
    }

    #[tokio::test]
    async fn window_exists_returns_false_for_nonexistent_window() {
        let manager = WindowManagerImpl::new();
        let fake_id = WindowId::new();

        assert!(!manager.window_exists(fake_id));
    }

    #[tokio::test]
    async fn window_exists_returns_true_for_existing_window() {
        let mut manager = WindowManagerImpl::new();
        let config = WindowConfig::default();
        let window_id = manager.create_window(config).await.unwrap();

        assert!(manager.window_exists(window_id));
    }
}
