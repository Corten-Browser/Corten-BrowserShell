//! Linux X11 window implementation
//!
//! This module provides an X11 window implementation following x11rb patterns.
//! The API surface matches what would be used with the x11rb crate for native
//! X11 window management.
//!
//! # x11rb Integration Notes
//!
//! When building with actual x11rb dependency:
//! - Replace stub connection with `x11rb::connect(None)?`
//! - Use `conn.create_window(...)` for window creation
//! - Use `conn.map_window(window_id)` for showing
//! - Use `conn.unmap_window(window_id)` for hiding
//! - Use `conn.configure_window(...)` for resize/move

use crate::{LinuxX11Handle, PlatformHandle, PlatformWindow};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicU32, Ordering};

/// Global counter for generating unique X11 window IDs
/// X11 XIDs typically start from a base value allocated by the server
static XID_COUNTER: AtomicU32 = AtomicU32::new(0x200001);

/// Mock X11 display connection pointer
static DISPLAY_PTR: AtomicU32 = AtomicU32::new(0x7F000000);

/// X11 window state flags (matching Xlib/xcb conventions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X11WindowState {
    /// Window is currently mapped (visible)
    pub mapped: bool,
    /// Window has input focus
    pub focused: bool,
    /// Window is minimized (iconic state)
    pub minimized: bool,
    /// Window is maximized
    pub maximized: bool,
    /// Window is fullscreen
    pub fullscreen: bool,
}

impl Default for X11WindowState {
    fn default() -> Self {
        Self {
            mapped: false,
            focused: false,
            minimized: false,
            maximized: false,
            fullscreen: false,
        }
    }
}

/// X11 window attributes (matching XSetWindowAttributes)
#[derive(Debug, Clone)]
pub struct X11WindowAttributes {
    /// Background pixel value
    pub background_pixel: u32,
    /// Border pixel value
    pub border_pixel: u32,
    /// Border width in pixels
    pub border_width: u32,
    /// Event mask for window events
    pub event_mask: u32,
    /// Override redirect flag
    pub override_redirect: bool,
    /// Colormap ID
    pub colormap: u32,
}

impl Default for X11WindowAttributes {
    fn default() -> Self {
        Self {
            background_pixel: 0xFFFFFF, // White
            border_pixel: 0x000000,     // Black
            border_width: 0,
            // ExposureMask | KeyPressMask | KeyReleaseMask | ButtonPressMask |
            // ButtonReleaseMask | StructureNotifyMask | FocusChangeMask
            event_mask: 0x0002_0033,
            override_redirect: false,
            colormap: 0, // CopyFromParent
        }
    }
}

/// X11 atom cache for common atoms
#[derive(Debug, Clone)]
pub struct X11Atoms {
    /// WM_DELETE_WINDOW atom
    pub wm_delete_window: u32,
    /// WM_PROTOCOLS atom
    pub wm_protocols: u32,
    /// _NET_WM_NAME atom
    pub net_wm_name: u32,
    /// _NET_WM_STATE atom
    pub net_wm_state: u32,
    /// _NET_WM_STATE_FULLSCREEN atom
    pub net_wm_state_fullscreen: u32,
    /// _NET_WM_STATE_MAXIMIZED_HORZ atom
    pub net_wm_state_maximized_horz: u32,
    /// _NET_WM_STATE_MAXIMIZED_VERT atom
    pub net_wm_state_maximized_vert: u32,
}

impl Default for X11Atoms {
    fn default() -> Self {
        // Mock atom values (real values come from XInternAtom)
        Self {
            wm_delete_window: 0x100,
            wm_protocols: 0x101,
            net_wm_name: 0x102,
            net_wm_state: 0x103,
            net_wm_state_fullscreen: 0x104,
            net_wm_state_maximized_horz: 0x105,
            net_wm_state_maximized_vert: 0x106,
        }
    }
}

/// Linux X11 window implementation
///
/// This implementation provides an API surface compatible with x11rb patterns.
/// In a full implementation, this would wrap an actual X11 connection and window.
#[derive(Debug)]
pub struct LinuxX11Window {
    /// X11 Window ID (XID)
    window_id: u32,
    /// X11 Display connection pointer (mock)
    display: usize,
    /// Screen number
    screen: i32,
    /// Visual ID used for the window
    visual_id: u32,
    /// Current window state
    state: X11WindowState,
    /// Window attributes
    attributes: X11WindowAttributes,
    /// Cached atoms
    atoms: X11Atoms,
    /// Window geometry
    geometry: X11Geometry,
    /// Window title
    title: String,
    /// Configuration options
    config: WindowConfigX11,
}

/// X11 window geometry
#[derive(Debug, Clone, Copy)]
pub struct X11Geometry {
    /// X position relative to parent
    pub x: i32,
    /// Y position relative to parent
    pub y: i32,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Depth (bits per pixel)
    pub depth: u8,
}

/// X11-specific window configuration
#[derive(Debug, Clone)]
pub struct WindowConfigX11 {
    /// Whether to use override redirect
    pub override_redirect: bool,
    /// Parent window ID (0 for root)
    pub parent: u32,
    /// Visual ID to use (0 for default)
    pub visual: u32,
    /// Window class (InputOutput or InputOnly)
    pub window_class: u32,
}

impl Default for WindowConfigX11 {
    fn default() -> Self {
        Self {
            override_redirect: false,
            parent: 0, // Root window
            visual: 0, // CopyFromParent
            window_class: 1, // InputOutput
        }
    }
}

impl LinuxX11Window {
    /// Create a new X11 window with extended configuration
    pub fn create_with_config(
        config: &WindowConfig,
        x11_config: &WindowConfigX11,
    ) -> Result<Self, WindowError> {
        let window_id = XID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let display = DISPLAY_PTR.load(Ordering::SeqCst) as usize;

        // Default screen and visual (would query from X server)
        let screen = 0;
        let visual_id = 0x21; // TrueColor visual

        log::debug!(
            "LinuxX11Window::create_with_config - XID: 0x{:X}, display: 0x{:X}, title: '{}'",
            window_id,
            display,
            config.title
        );

        let geometry = X11Geometry {
            x: config.x.unwrap_or(0),
            y: config.y.unwrap_or(0),
            width: config.width,
            height: config.height,
            depth: 24, // 24-bit color
        };

        let mut attributes = X11WindowAttributes::default();
        attributes.override_redirect = x11_config.override_redirect;

        Ok(Self {
            window_id,
            display,
            screen,
            visual_id,
            state: X11WindowState::default(),
            attributes,
            atoms: X11Atoms::default(),
            geometry,
            title: config.title.clone(),
            config: x11_config.clone(),
        })
    }

    /// Get the X11 window ID (XID)
    pub fn xid(&self) -> u32 {
        self.window_id
    }

    /// Get the display connection
    pub fn display(&self) -> usize {
        self.display
    }

    /// Get the current window state
    pub fn state(&self) -> &X11WindowState {
        &self.state
    }

    /// Get window geometry
    pub fn geometry(&self) -> &X11Geometry {
        &self.geometry
    }

    /// Set window property (XChangeProperty pattern)
    pub fn set_property(&self, atom: u32, property_type: u32, data: &[u8]) -> Result<(), WindowError> {
        log::debug!(
            "LinuxX11Window::set_property - XID: 0x{:X}, atom: 0x{:X}, type: 0x{:X}, len: {}",
            self.window_id,
            atom,
            property_type,
            data.len()
        );
        Ok(())
    }

    /// Set window title using _NET_WM_NAME
    pub fn set_title(&mut self, title: &str) -> Result<(), WindowError> {
        log::debug!(
            "LinuxX11Window::set_title - XID: 0x{:X}, title: '{}'",
            self.window_id,
            title
        );
        self.title = title.to_string();
        // In real implementation: XChangeProperty with _NET_WM_NAME atom
        Ok(())
    }

    /// Set fullscreen state using _NET_WM_STATE
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), WindowError> {
        log::debug!(
            "LinuxX11Window::set_fullscreen - XID: 0x{:X}, fullscreen: {}",
            self.window_id,
            fullscreen
        );
        self.state.fullscreen = fullscreen;
        // In real implementation: Send _NET_WM_STATE client message
        Ok(())
    }

    /// Set maximized state
    pub fn set_maximized(&mut self, maximized: bool) -> Result<(), WindowError> {
        log::debug!(
            "LinuxX11Window::set_maximized - XID: 0x{:X}, maximized: {}",
            self.window_id,
            maximized
        );
        self.state.maximized = maximized;
        Ok(())
    }

    /// Raise window to top of stacking order
    pub fn raise(&self) -> Result<(), WindowError> {
        log::debug!("LinuxX11Window::raise - XID: 0x{:X}", self.window_id);
        // In real implementation: XRaiseWindow or XConfigureWindow with CWStackMode
        Ok(())
    }

    /// Lower window to bottom of stacking order
    pub fn lower(&self) -> Result<(), WindowError> {
        log::debug!("LinuxX11Window::lower - XID: 0x{:X}", self.window_id);
        Ok(())
    }

    /// Set window decorations hint (via _MOTIF_WM_HINTS)
    pub fn set_decorations(&self, decorations: bool) -> Result<(), WindowError> {
        log::debug!(
            "LinuxX11Window::set_decorations - XID: 0x{:X}, decorations: {}",
            self.window_id,
            decorations
        );
        Ok(())
    }

    /// Flush pending X11 requests (XFlush pattern)
    pub fn flush(&self) -> Result<(), WindowError> {
        log::trace!("LinuxX11Window::flush - XID: 0x{:X}", self.window_id);
        Ok(())
    }

    /// Sync with X server (XSync pattern)
    pub fn sync(&self, discard: bool) -> Result<(), WindowError> {
        log::trace!(
            "LinuxX11Window::sync - XID: 0x{:X}, discard: {}",
            self.window_id,
            discard
        );
        Ok(())
    }
}

impl PlatformWindow for LinuxX11Window {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        Self::create_with_config(config, &WindowConfigX11::default())
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxX11Window::destroy - XID: 0x{:X}", self.window_id);
        // In real implementation: XDestroyWindow
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxX11Window::show - XID: 0x{:X}", self.window_id);
        self.state.mapped = true;
        // In real implementation: XMapWindow
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxX11Window::hide - XID: 0x{:X}", self.window_id);
        self.state.mapped = false;
        // In real implementation: XUnmapWindow
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxX11Window::resize - XID: 0x{:X}, size: {}x{}",
            self.window_id,
            width,
            height
        );
        self.geometry.width = width;
        self.geometry.height = height;
        // In real implementation: XResizeWindow or XConfigureWindow
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxX11Window::move_to - XID: 0x{:X}, position: ({}, {})",
            self.window_id,
            x,
            y
        );
        self.geometry.x = x;
        self.geometry.y = y;
        // In real implementation: XMoveWindow or XConfigureWindow
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxX11Window::focus - XID: 0x{:X}", self.window_id);
        self.state.focused = true;
        // In real implementation: XSetInputFocus with RevertToParent
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::LinuxX11(LinuxX11Handle {
            window: self.window_id,
            display: self.display,
            screen: self.screen,
            visual_id: self.visual_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlatformWindow;

    #[test]
    fn test_x11_window_create() {
        let config = WindowConfig::default();
        let result = LinuxX11Window::create(&config);
        assert!(result.is_ok());
        let window = result.unwrap();
        assert!(window.xid() > 0);
    }

    #[test]
    fn test_x11_window_operations() {
        let config = WindowConfig::default();
        let mut window = LinuxX11Window::create(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.state().mapped);

        assert!(window.hide().is_ok());
        assert!(!window.state().mapped);

        assert!(window.resize(800, 600).is_ok());
        assert_eq!(window.geometry().width, 800);
        assert_eq!(window.geometry().height, 600);

        assert!(window.move_to(100, 200).is_ok());
        assert_eq!(window.geometry().x, 100);
        assert_eq!(window.geometry().y, 200);

        assert!(window.focus().is_ok());
        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_x11_window_handle() {
        let config = WindowConfig::default();
        let window = LinuxX11Window::create(&config).unwrap();
        let handle = window.get_handle();

        assert!(handle.is_x11());
        match handle {
            PlatformHandle::LinuxX11(h) => {
                assert_eq!(h.window, window.xid());
                assert_eq!(h.screen, 0);
            }
            _ => panic!("Expected X11 handle"),
        }
    }

    #[test]
    fn test_x11_window_fullscreen() {
        let config = WindowConfig::default();
        let mut window = LinuxX11Window::create(&config).unwrap();

        assert!(!window.state().fullscreen);
        assert!(window.set_fullscreen(true).is_ok());
        assert!(window.state().fullscreen);
        assert!(window.set_fullscreen(false).is_ok());
        assert!(!window.state().fullscreen);
    }

    #[test]
    fn test_x11_window_title() {
        let config = WindowConfig {
            title: "Test Window".to_string(),
            ..Default::default()
        };
        let mut window = LinuxX11Window::create(&config).unwrap();

        assert_eq!(window.title, "Test Window");
        assert!(window.set_title("New Title").is_ok());
        assert_eq!(window.title, "New Title");
    }

    #[test]
    fn test_x11_extended_config() {
        let config = WindowConfig::default();
        let x11_config = WindowConfigX11 {
            override_redirect: true,
            parent: 0x100,
            visual: 0x21,
            window_class: 1,
        };

        let window = LinuxX11Window::create_with_config(&config, &x11_config).unwrap();
        assert!(window.attributes.override_redirect);
    }
}
