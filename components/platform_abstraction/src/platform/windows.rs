//! Windows platform window implementation
//!
//! This is a stub implementation that provides mock functionality for testing.
//! Full Win32 API integration will be implemented in later phases.

use crate::{PlatformHandle, PlatformWindow, WindowsHandle};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global counter for generating unique window handles
static HWND_COUNTER: AtomicUsize = AtomicUsize::new(0x1000);

/// Windows window implementation (stub)
///
/// This is a minimal stub implementation that satisfies the PlatformWindow trait
/// for testing purposes. It generates mock HWND values and logs operations.
#[derive(Debug)]
pub struct WindowsWindow {
    /// Mock HWND value
    hwnd: usize,
    /// Current window state
    visible: bool,
    /// Window dimensions
    width: u32,
    height: u32,
    /// Window position
    x: i32,
    y: i32,
}

impl PlatformWindow for WindowsWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        let hwnd = HWND_COUNTER.fetch_add(0x10, Ordering::SeqCst);

        log::debug!(
            "WindowsWindow::create - HWND: 0x{:X}, title: '{}', size: {}x{}",
            hwnd,
            config.title,
            config.width,
            config.height
        );

        Ok(Self {
            hwnd,
            visible: false,
            width: config.width,
            height: config.height,
            x: config.x.unwrap_or(0),
            y: config.y.unwrap_or(0),
        })
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::destroy - HWND: 0x{:X}", self.hwnd);
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::show - HWND: 0x{:X}", self.hwnd);
        self.visible = true;
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::hide - HWND: 0x{:X}", self.hwnd);
        self.visible = false;
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::resize - HWND: 0x{:X}, size: {}x{}",
            self.hwnd,
            width,
            height
        );
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "WindowsWindow::move_to - HWND: 0x{:X}, position: ({}, {})",
            self.hwnd,
            x,
            y
        );
        self.x = x;
        self.y = y;
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("WindowsWindow::focus - HWND: 0x{:X}", self.hwnd);
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::Windows(WindowsHandle { hwnd: self.hwnd })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_window_create() {
        let config = WindowConfig::default();
        let result = WindowsWindow::create(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_windows_window_operations() {
        let config = WindowConfig::default();
        let mut window = WindowsWindow::create(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.hide().is_ok());
        assert!(window.resize(800, 600).is_ok());
        assert!(window.move_to(100, 200).is_ok());
        assert!(window.focus().is_ok());
        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_windows_window_handle() {
        let config = WindowConfig::default();
        let window = WindowsWindow::create(&config).unwrap();
        let handle = window.get_handle();

        match handle {
            PlatformHandle::Windows(h) => assert!(h.hwnd >= 0x1000),
            _ => panic!("Expected Windows handle"),
        }
    }
}
