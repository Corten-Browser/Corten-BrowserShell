//! Linux platform window implementation (X11/Wayland)
//!
//! This is a stub implementation that provides mock functionality for testing.
//! Full X11/Wayland integration will be implemented in later phases.

use crate::{LinuxHandle, PlatformHandle, PlatformWindow};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicU32, Ordering};

/// Global counter for generating unique window IDs
static WINDOW_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Linux window implementation (stub)
///
/// This is a minimal stub implementation that satisfies the PlatformWindow trait
/// for testing purposes. It generates mock window IDs and logs operations.
#[derive(Debug)]
pub struct LinuxWindow {
    /// Mock window ID
    window_id: u32,
    /// Current window state
    visible: bool,
    /// Window dimensions
    width: u32,
    height: u32,
    /// Window position
    x: i32,
    y: i32,
}

impl PlatformWindow for LinuxWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        let window_id = WINDOW_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        log::debug!(
            "LinuxWindow::create - ID: {}, title: '{}', size: {}x{}",
            window_id,
            config.title,
            config.width,
            config.height
        );

        Ok(Self {
            window_id,
            visible: false,
            width: config.width,
            height: config.height,
            x: config.x.unwrap_or(0),
            y: config.y.unwrap_or(0),
        })
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWindow::destroy - ID: {}", self.window_id);
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWindow::show - ID: {}", self.window_id);
        self.visible = true;
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWindow::hide - ID: {}", self.window_id);
        self.visible = false;
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWindow::resize - ID: {}, size: {}x{}",
            self.window_id,
            width,
            height
        );
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWindow::move_to - ID: {}, position: ({}, {})",
            self.window_id,
            x,
            y
        );
        self.x = x;
        self.y = y;
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWindow::focus - ID: {}", self.window_id);
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::Linux(LinuxHandle {
            window: self.window_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_window_create() {
        let config = WindowConfig::default();
        let result = LinuxWindow::create(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_linux_window_operations() {
        let config = WindowConfig::default();
        let mut window = LinuxWindow::create(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.hide().is_ok());
        assert!(window.resize(800, 600).is_ok());
        assert!(window.move_to(100, 200).is_ok());
        assert!(window.focus().is_ok());
        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_linux_window_handle() {
        let config = WindowConfig::default();
        let window = LinuxWindow::create(&config).unwrap();
        let handle = window.get_handle();

        match handle {
            PlatformHandle::Linux(h) => assert!(h.window > 0),
            _ => panic!("Expected Linux handle"),
        }
    }
}
