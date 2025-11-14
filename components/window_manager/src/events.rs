// @implements: REQ-006
//! Platform event handling module

use shared_types::{WindowError, window::PlatformEvent};
use crate::window_state::WindowState;

/// Event handler for platform window events
pub struct EventHandler;

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        Self
    }

    /// Handle a platform event
    ///
    /// # Arguments
    ///
    /// * `window_state` - Mutable reference to window state
    /// * `event` - Platform event to handle
    ///
    /// # Returns
    ///
    /// Ok if event handled successfully, error otherwise
    ///
    /// # Errors
    ///
    /// Returns error if event cannot be handled or state update fails
    pub fn handle_event(
        &self,
        window_state: &mut WindowState,
        event: PlatformEvent,
    ) -> Result<(), WindowError> {
        match event {
            PlatformEvent::Resized { width, height } => {
                window_state.update_size(width, height);
            }
            PlatformEvent::Moved { x, y } => {
                window_state.update_position(x, y);
            }
            PlatformEvent::CloseRequested => {
                // Close request is handled by the window manager
                // This event is informational only
            }
            PlatformEvent::Focused => {
                window_state.set_focused(true);
            }
            PlatformEvent::Unfocused => {
                window_state.set_focused(false);
            }
            PlatformEvent::KeyboardInput { .. } => {
                // Keyboard events are forwarded to the active tab
                // Event routing is handled by message bus
            }
            PlatformEvent::MouseInput { .. } => {
                // Mouse events are forwarded to the active tab
                // Event routing is handled by message bus
            }
        }

        Ok(())
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::window::{WindowId, WindowConfig};

    #[test]
    fn event_handler_new_creates_instance() {
        let handler = EventHandler::new();
        // Just verify it can be created
        let _ = handler;
    }

    #[test]
    fn event_handler_default_creates_instance() {
        let handler = EventHandler::default();
        let _ = handler;
    }

    #[test]
    fn handle_event_resized_updates_window_size() {
        let handler = EventHandler::new();
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let event = PlatformEvent::Resized { width: 1920, height: 1080 };
        let result = handler.handle_event(&mut state, event);

        assert!(result.is_ok());
        let window = state.as_window();
        assert_eq!(window.config.width, 1920);
        assert_eq!(window.config.height, 1080);
    }

    #[test]
    fn handle_event_moved_updates_window_position() {
        let handler = EventHandler::new();
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let event = PlatformEvent::Moved { x: 100, y: 200 };
        let result = handler.handle_event(&mut state, event);

        assert!(result.is_ok());
        let window = state.as_window();
        assert_eq!(window.config.x, Some(100));
        assert_eq!(window.config.y, Some(200));
    }

    #[test]
    fn handle_event_focused_sets_focus() {
        let handler = EventHandler::new();
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let event = PlatformEvent::Focused;
        let result = handler.handle_event(&mut state, event);

        assert!(result.is_ok());
        let window = state.as_window();
        assert!(window.is_focused);
    }

    #[test]
    fn handle_event_unfocused_removes_focus() {
        let handler = EventHandler::new();
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        // First set focused
        state.set_focused(true);
        assert!(state.as_window().is_focused);

        // Then unfocus via event
        let event = PlatformEvent::Unfocused;
        let result = handler.handle_event(&mut state, event);

        assert!(result.is_ok());
        let window = state.as_window();
        assert!(!window.is_focused);
    }

    #[test]
    fn handle_event_close_requested_succeeds() {
        let handler = EventHandler::new();
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let event = PlatformEvent::CloseRequested;
        let result = handler.handle_event(&mut state, event);

        assert!(result.is_ok());
    }

    #[test]
    fn handle_event_keyboard_input_succeeds() {
        let handler = EventHandler::new();
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let event = PlatformEvent::KeyboardInput {
            key: "a".to_string(),
            pressed: true,
        };
        let result = handler.handle_event(&mut state, event);

        assert!(result.is_ok());
    }

    #[test]
    fn handle_event_mouse_input_succeeds() {
        let handler = EventHandler::new();
        let id = WindowId::new();
        let config = WindowConfig::default();
        let mut state = WindowState::new(id, config);

        let event = PlatformEvent::MouseInput {
            x: 100,
            y: 200,
            button: "left".to_string(),
            pressed: true,
        };
        let result = handler.handle_event(&mut state, event);

        assert!(result.is_ok());
    }
}
