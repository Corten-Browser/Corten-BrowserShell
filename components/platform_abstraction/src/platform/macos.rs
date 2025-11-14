//! macOS platform window implementation
//!
//! This is a stub implementation that provides mock functionality for testing.
//! Full Cocoa/AppKit integration will be implemented in later phases.

use crate::{MacOSHandle, PlatformHandle, PlatformWindow};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global counter for generating unique NSWindow pointers
static NSWINDOW_COUNTER: AtomicUsize = AtomicUsize::new(0x7FFF0000);

/// macOS window implementation (stub)
///
/// This is a minimal stub implementation that satisfies the PlatformWindow trait
/// for testing purposes. It generates mock NSWindow pointer values and logs operations.
#[derive(Debug)]
pub struct MacWindow {
    /// Mock NSWindow pointer value
    ns_window: usize,
    /// Current window state
    visible: bool,
    /// Window dimensions
    width: u32,
    height: u32,
    /// Window position
    x: i32,
    y: i32,
}

impl PlatformWindow for MacWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        let ns_window = NSWINDOW_COUNTER.fetch_add(0x1000, Ordering::SeqCst);

        log::debug!(
            "MacWindow::create - NSWindow: 0x{:X}, title: '{}', size: {}x{}",
            ns_window,
            config.title,
            config.width,
            config.height
        );

        Ok(Self {
            ns_window,
            visible: false,
            width: config.width,
            height: config.height,
            x: config.x.unwrap_or(0),
            y: config.y.unwrap_or(0),
        })
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::destroy - NSWindow: 0x{:X}", self.ns_window);
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::show - NSWindow: 0x{:X}", self.ns_window);
        self.visible = true;
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::hide - NSWindow: 0x{:X}", self.ns_window);
        self.visible = false;
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::resize - NSWindow: 0x{:X}, size: {}x{}",
            self.ns_window,
            width,
            height
        );
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "MacWindow::move_to - NSWindow: 0x{:X}, position: ({}, {})",
            self.ns_window,
            x,
            y
        );
        self.x = x;
        self.y = y;
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("MacWindow::focus - NSWindow: 0x{:X}", self.ns_window);
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::MacOS(MacOSHandle {
            ns_window: self.ns_window,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_window_create() {
        let config = WindowConfig::default();
        let result = MacWindow::create(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mac_window_operations() {
        let config = WindowConfig::default();
        let mut window = MacWindow::create(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.hide().is_ok());
        assert!(window.resize(800, 600).is_ok());
        assert!(window.move_to(100, 200).is_ok());
        assert!(window.focus().is_ok());
        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_mac_window_handle() {
        let config = WindowConfig::default();
        let window = MacWindow::create(&config).unwrap();
        let handle = window.get_handle();

        match handle {
            PlatformHandle::MacOS(h) => assert!(h.ns_window >= 0x7FFF0000),
            _ => panic!("Expected MacOS handle"),
        }
    }
}
