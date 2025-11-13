// @validates: REQ-003
//! Unit tests for platform event translation

use shared_types::window::PlatformEvent;

#[cfg(test)]
mod event_translation_tests {
    use super::*;

    #[test]
    fn test_translate_resize_event() {
        // RED: Event translation not implemented

        // Given a native platform resize event
        // When we translate it to PlatformEvent
        // Then it should be PlatformEvent::Resized

        panic!("Event translation not implemented yet");
    }

    #[test]
    fn test_translate_keyboard_event() {
        // RED: Keyboard event translation not implemented

        // Given a native keyboard event
        // When we translate it
        // Then it should be PlatformEvent::KeyboardInput

        panic!("Keyboard event translation not implemented yet");
    }

    #[test]
    fn test_translate_mouse_event() {
        // RED: Mouse event translation not implemented

        // Given a native mouse event
        // When we translate it
        // Then it should be PlatformEvent::MouseInput

        panic!("Mouse event translation not implemented yet");
    }

    #[test]
    fn test_translate_close_event() {
        // RED: Close event translation not implemented

        // Given a window close request
        // When we translate it
        // Then it should be PlatformEvent::CloseRequested

        panic!("Close event translation not implemented yet");
    }
}
