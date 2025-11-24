//! Platform-specific window implementations
//!
//! This module contains implementations for each supported platform.
//! It provides both platform detection and platform-specific window types.
//!
//! # Architecture
//!
//! - **detection**: Runtime platform and display server detection
//! - **linux_x11**: X11 window implementation (x11rb patterns)
//! - **linux_wayland**: Wayland window implementation (wayland-client patterns)
//! - **windows**: Win32 window implementation (windows-rs patterns)
//! - **macos**: AppKit window implementation (cocoa patterns)
//!
//! # Usage
//!
//! ```rust,ignore
//! use platform_abstraction::platform::{
//!     detection::{current_platform, detect_display_server, Platform, DisplayServer},
//!     create_platform_window,
//! };
//!
//! let info = detection::get_platform_info();
//! let window = create_platform_window(&config)?;
//! ```

pub mod detection;

// Platform-specific modules (always available for testing)
pub mod linux_x11;
pub mod linux_wayland;
pub mod windows;
pub mod macos;

// Re-export detection types at module level
pub use detection::{
    current_platform, detect_display_server, get_platform_info, preferred_display_server,
    supports_display_server, DisplayServer, Platform, PlatformDetails, PlatformInfo,
};

// Re-export platform-specific window types
pub use linux_x11::{
    LinuxX11Window, WindowConfigX11, X11Atoms, X11Geometry, X11WindowAttributes, X11WindowState,
};
pub use linux_wayland::{
    DecorationMode, LinuxWaylandWindow, ToplevelCapabilities, WaylandConfigureState,
    WaylandGeometry, WaylandSurfaceConfig, WaylandTiledState, WaylandWindowState,
};
pub use macos::{
    AppearanceMode, BackingStoreType, CollectionBehavior, MacWindow, MacWindowState, NSRect,
    WindowStyleMask,
};
pub use windows::{DwmAttributes, WindowRect, WindowStyle, WindowsWindow, WindowsWindowState};

use crate::{PlatformHandle, PlatformWindow, StubHandle};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for stub window IDs
static STUB_WINDOW_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A generic platform window that wraps the appropriate platform implementation
#[derive(Debug)]
pub enum GenericPlatformWindow {
    /// X11 window on Linux
    LinuxX11(LinuxX11Window),
    /// Wayland window on Linux
    LinuxWayland(LinuxWaylandWindow),
    /// Win32 window on Windows
    Windows(WindowsWindow),
    /// AppKit window on macOS
    MacOS(MacWindow),
    /// Stub window for testing/headless
    Stub(StubWindow),
}

/// Stub window implementation for testing and headless environments
#[derive(Debug)]
pub struct StubWindow {
    id: u64,
    visible: bool,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    title: String,
}

impl StubWindow {
    /// Create a new stub window
    pub fn new(config: &WindowConfig) -> Self {
        let id = STUB_WINDOW_COUNTER.fetch_add(1, Ordering::SeqCst);
        log::debug!(
            "StubWindow::new - id: {}, title: '{}', size: {}x{}",
            id,
            config.title,
            config.width,
            config.height
        );
        Self {
            id,
            visible: false,
            width: config.width,
            height: config.height,
            x: config.x.unwrap_or(0),
            y: config.y.unwrap_or(0),
            title: config.title.clone(),
        }
    }

    /// Get the stub window ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get size
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get position
    pub fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl PlatformWindow for StubWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        Ok(Self::new(config))
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("StubWindow::destroy - id: {}", self.id);
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("StubWindow::show - id: {}", self.id);
        self.visible = true;
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("StubWindow::hide - id: {}", self.id);
        self.visible = false;
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "StubWindow::resize - id: {}, size: {}x{}",
            self.id,
            width,
            height
        );
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "StubWindow::move_to - id: {}, position: ({}, {})",
            self.id,
            x,
            y
        );
        self.x = x;
        self.y = y;
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("StubWindow::focus - id: {}", self.id);
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::Stub(StubHandle { id: self.id })
    }
}

impl GenericPlatformWindow {
    /// Show the window
    pub fn show(&mut self) -> Result<(), WindowError> {
        match self {
            Self::LinuxX11(w) => w.show(),
            Self::LinuxWayland(w) => w.show(),
            Self::Windows(w) => w.show(),
            Self::MacOS(w) => w.show(),
            Self::Stub(w) => w.show(),
        }
    }

    /// Hide the window
    pub fn hide(&mut self) -> Result<(), WindowError> {
        match self {
            Self::LinuxX11(w) => w.hide(),
            Self::LinuxWayland(w) => w.hide(),
            Self::Windows(w) => w.hide(),
            Self::MacOS(w) => w.hide(),
            Self::Stub(w) => w.hide(),
        }
    }

    /// Resize the window
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        match self {
            Self::LinuxX11(w) => w.resize(width, height),
            Self::LinuxWayland(w) => w.resize(width, height),
            Self::Windows(w) => w.resize(width, height),
            Self::MacOS(w) => w.resize(width, height),
            Self::Stub(w) => w.resize(width, height),
        }
    }

    /// Move the window
    pub fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        match self {
            Self::LinuxX11(w) => w.move_to(x, y),
            Self::LinuxWayland(w) => w.move_to(x, y),
            Self::Windows(w) => w.move_to(x, y),
            Self::MacOS(w) => w.move_to(x, y),
            Self::Stub(w) => w.move_to(x, y),
        }
    }

    /// Focus the window
    pub fn focus(&mut self) -> Result<(), WindowError> {
        match self {
            Self::LinuxX11(w) => w.focus(),
            Self::LinuxWayland(w) => w.focus(),
            Self::Windows(w) => w.focus(),
            Self::MacOS(w) => w.focus(),
            Self::Stub(w) => w.focus(),
        }
    }

    /// Destroy the window
    pub fn destroy(&mut self) -> Result<(), WindowError> {
        match self {
            Self::LinuxX11(w) => w.destroy(),
            Self::LinuxWayland(w) => w.destroy(),
            Self::Windows(w) => w.destroy(),
            Self::MacOS(w) => w.destroy(),
            Self::Stub(w) => w.destroy(),
        }
    }

    /// Get the platform handle
    pub fn get_handle(&self) -> PlatformHandle {
        match self {
            Self::LinuxX11(w) => w.get_handle(),
            Self::LinuxWayland(w) => w.get_handle(),
            Self::Windows(w) => w.get_handle(),
            Self::MacOS(w) => w.get_handle(),
            Self::Stub(w) => w.get_handle(),
        }
    }
}

/// Create a platform window based on the detected platform
///
/// This function automatically detects the current platform and display server,
/// then creates the appropriate window type.
///
/// # Arguments
///
/// * `config` - Window configuration
///
/// # Returns
///
/// * `Ok(GenericPlatformWindow)` - Successfully created window
/// * `Err(WindowError)` - Window creation failed
pub fn create_platform_window(config: &WindowConfig) -> Result<GenericPlatformWindow, WindowError> {
    let platform = current_platform();
    let display_server = detect_display_server();

    log::debug!(
        "create_platform_window - platform: {:?}, display_server: {:?}",
        platform,
        display_server
    );

    match (platform, display_server) {
        (Platform::Linux, DisplayServer::X11) | (Platform::Linux, DisplayServer::XWayland) => {
            LinuxX11Window::create(config).map(GenericPlatformWindow::LinuxX11)
        }
        (Platform::Linux, DisplayServer::Wayland) => {
            LinuxWaylandWindow::create(config).map(GenericPlatformWindow::LinuxWayland)
        }
        (Platform::Windows, _) => {
            WindowsWindow::create(config).map(GenericPlatformWindow::Windows)
        }
        (Platform::MacOS, _) => MacWindow::create(config).map(GenericPlatformWindow::MacOS),
        (Platform::Linux, DisplayServer::Headless) | (_, DisplayServer::Unknown) | (_, _) => {
            // Fall back to stub for headless or unknown platforms
            log::warn!(
                "Using stub window for platform {:?} with display server {:?}",
                platform,
                display_server
            );
            Ok(GenericPlatformWindow::Stub(StubWindow::new(config)))
        }
    }
}

/// Create a platform window for a specific display server
///
/// This allows explicit control over which window type to create.
pub fn create_window_for_display_server(
    config: &WindowConfig,
    display_server: DisplayServer,
) -> Result<GenericPlatformWindow, WindowError> {
    log::debug!(
        "create_window_for_display_server - display_server: {:?}",
        display_server
    );

    match display_server {
        DisplayServer::X11 | DisplayServer::XWayland => {
            LinuxX11Window::create(config).map(GenericPlatformWindow::LinuxX11)
        }
        DisplayServer::Wayland => {
            LinuxWaylandWindow::create(config).map(GenericPlatformWindow::LinuxWayland)
        }
        DisplayServer::WindowsDwm => {
            WindowsWindow::create(config).map(GenericPlatformWindow::Windows)
        }
        DisplayServer::MacOSQuartz => MacWindow::create(config).map(GenericPlatformWindow::MacOS),
        DisplayServer::Headless | DisplayServer::Unknown => {
            Ok(GenericPlatformWindow::Stub(StubWindow::new(config)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_platform_window() {
        let config = WindowConfig::default();
        let result = create_platform_window(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_window_operations() {
        let config = WindowConfig::default();
        let mut window = create_platform_window(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.resize(800, 600).is_ok());
        assert!(window.move_to(100, 200).is_ok());
        assert!(window.focus().is_ok());
        assert!(window.hide().is_ok());
        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_stub_window() {
        let config = WindowConfig::default();
        let mut window = StubWindow::new(&config);

        assert!(window.show().is_ok());
        assert!(window.is_visible());
        assert!(window.hide().is_ok());
        assert!(!window.is_visible());
    }

    #[test]
    fn test_create_window_for_display_server() {
        let config = WindowConfig::default();

        // Test X11
        let x11 = create_window_for_display_server(&config, DisplayServer::X11);
        assert!(x11.is_ok());
        assert!(matches!(x11.unwrap(), GenericPlatformWindow::LinuxX11(_)));

        // Test Wayland
        let wayland = create_window_for_display_server(&config, DisplayServer::Wayland);
        assert!(wayland.is_ok());
        assert!(matches!(
            wayland.unwrap(),
            GenericPlatformWindow::LinuxWayland(_)
        ));

        // Test Windows
        let win = create_window_for_display_server(&config, DisplayServer::WindowsDwm);
        assert!(win.is_ok());
        assert!(matches!(win.unwrap(), GenericPlatformWindow::Windows(_)));

        // Test macOS
        let mac = create_window_for_display_server(&config, DisplayServer::MacOSQuartz);
        assert!(mac.is_ok());
        assert!(matches!(mac.unwrap(), GenericPlatformWindow::MacOS(_)));

        // Test Headless
        let headless = create_window_for_display_server(&config, DisplayServer::Headless);
        assert!(headless.is_ok());
        assert!(matches!(headless.unwrap(), GenericPlatformWindow::Stub(_)));
    }

    #[test]
    fn test_platform_handle_from_generic() {
        let config = WindowConfig::default();

        let x11 = create_window_for_display_server(&config, DisplayServer::X11).unwrap();
        assert!(x11.get_handle().is_x11());

        let wayland = create_window_for_display_server(&config, DisplayServer::Wayland).unwrap();
        assert!(wayland.get_handle().is_wayland());

        let win = create_window_for_display_server(&config, DisplayServer::WindowsDwm).unwrap();
        assert!(win.get_handle().is_windows());

        let mac = create_window_for_display_server(&config, DisplayServer::MacOSQuartz).unwrap();
        assert!(mac.get_handle().is_macos());
    }
}
