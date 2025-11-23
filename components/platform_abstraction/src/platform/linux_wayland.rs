//! Linux Wayland window implementation
//!
//! This module provides a Wayland window implementation following wayland-client patterns.
//! The API surface matches what would be used with the wayland-client crate for native
//! Wayland window management.
//!
//! # wayland-client Integration Notes
//!
//! When building with actual wayland-client dependency:
//! - Replace stub display with `Display::connect_to_env()?`
//! - Use `compositor.create_surface()` for surface creation
//! - Use `xdg_shell.get_xdg_surface()` for shell surface
//! - Use `xdg_surface.get_toplevel()` for toplevel window
//! - Frame callbacks for synchronized drawing

use crate::{LinuxWaylandHandle, PlatformHandle, PlatformWindow};
use shared_types::{WindowConfig, WindowError};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Global counter for generating unique Wayland surface IDs
static SURFACE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Mock Wayland display pointer
static DISPLAY_PTR: AtomicUsize = AtomicUsize::new(0x7F100000);

/// Mock XDG toplevel pointer
static XDG_TOPLEVEL_PTR: AtomicUsize = AtomicUsize::new(0x7F200000);

/// Wayland window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaylandWindowState {
    /// Surface is committed and visible
    pub mapped: bool,
    /// Window has keyboard focus
    pub focused: bool,
    /// Window is activated (has user attention)
    pub activated: bool,
    /// Window is maximized
    pub maximized: bool,
    /// Window is fullscreen
    pub fullscreen: bool,
    /// Window is being resized
    pub resizing: bool,
    /// Window is being moved
    pub tiled: WaylandTiledState,
}

/// Wayland tiled state (for tiling window managers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaylandTiledState {
    /// Tiled on left edge
    pub left: bool,
    /// Tiled on right edge
    pub right: bool,
    /// Tiled on top edge
    pub top: bool,
    /// Tiled on bottom edge
    pub bottom: bool,
}

impl Default for WaylandWindowState {
    fn default() -> Self {
        Self {
            mapped: false,
            focused: false,
            activated: false,
            maximized: false,
            fullscreen: false,
            resizing: false,
            tiled: WaylandTiledState::default(),
        }
    }
}

/// XDG toplevel decoration mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationMode {
    /// Client-side decorations (application draws borders)
    ClientSide,
    /// Server-side decorations (compositor draws borders)
    ServerSide,
}

/// XDG toplevel capabilities reported by compositor
#[derive(Debug, Clone, Copy)]
pub struct ToplevelCapabilities {
    /// Can maximize
    pub maximize: bool,
    /// Can minimize
    pub minimize: bool,
    /// Can go fullscreen
    pub fullscreen: bool,
    /// Can resize
    pub resize: bool,
}

impl Default for ToplevelCapabilities {
    fn default() -> Self {
        Self {
            maximize: true,
            minimize: true,
            fullscreen: true,
            resize: true,
        }
    }
}

/// Wayland surface configuration (from xdg_toplevel.configure)
#[derive(Debug, Clone)]
pub struct WaylandSurfaceConfig {
    /// Suggested width (0 = compositor doesn't care)
    pub width: u32,
    /// Suggested height (0 = compositor doesn't care)
    pub height: u32,
    /// Configure serial for acking
    pub serial: u32,
    /// States active in this configuration
    pub states: Vec<WaylandConfigureState>,
}

/// States that can be set during configure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaylandConfigureState {
    /// Window is maximized
    Maximized,
    /// Window is fullscreen
    Fullscreen,
    /// Window is being resized
    Resizing,
    /// Window is activated
    Activated,
    /// Window is tiled left
    TiledLeft,
    /// Window is tiled right
    TiledRight,
    /// Window is tiled top
    TiledTop,
    /// Window is tiled bottom
    TiledBottom,
}

/// Linux Wayland window implementation
///
/// This implementation provides an API surface compatible with wayland-client patterns.
/// In a full implementation, this would wrap actual Wayland protocol objects.
#[derive(Debug)]
pub struct LinuxWaylandWindow {
    /// Wayland surface ID
    surface_id: u32,
    /// Wayland display connection pointer (mock)
    display: usize,
    /// XDG toplevel handle
    xdg_toplevel: usize,
    /// Current window state
    state: WaylandWindowState,
    /// Window title
    title: String,
    /// Application ID (for desktop integration)
    app_id: String,
    /// Current geometry
    geometry: WaylandGeometry,
    /// Pending configure serial
    pending_configure_serial: Option<u32>,
    /// Decoration mode
    decoration_mode: DecorationMode,
    /// Toplevel capabilities
    capabilities: ToplevelCapabilities,
    /// Minimum size constraint
    min_size: Option<(u32, u32)>,
    /// Maximum size constraint
    max_size: Option<(u32, u32)>,
}

/// Wayland window geometry
#[derive(Debug, Clone, Copy)]
pub struct WaylandGeometry {
    /// Logical width
    pub width: u32,
    /// Logical height
    pub height: u32,
    /// Scale factor (for HiDPI)
    pub scale: u32,
    /// Buffer transform (rotation)
    pub transform: u32,
}

impl Default for WaylandGeometry {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            scale: 1,
            transform: 0, // Normal orientation
        }
    }
}

impl LinuxWaylandWindow {
    /// Create a new Wayland window with extended options
    pub fn create_with_app_id(config: &WindowConfig, app_id: &str) -> Result<Self, WindowError> {
        let surface_id = SURFACE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let display = DISPLAY_PTR.load(Ordering::SeqCst);
        let xdg_toplevel = XDG_TOPLEVEL_PTR.fetch_add(0x1000, Ordering::SeqCst);

        log::debug!(
            "LinuxWaylandWindow::create_with_app_id - surface: {}, display: 0x{:X}, title: '{}', app_id: '{}'",
            surface_id,
            display,
            config.title,
            app_id
        );

        Ok(Self {
            surface_id,
            display,
            xdg_toplevel,
            state: WaylandWindowState::default(),
            title: config.title.clone(),
            app_id: app_id.to_string(),
            geometry: WaylandGeometry {
                width: config.width,
                height: config.height,
                ..Default::default()
            },
            pending_configure_serial: None,
            decoration_mode: DecorationMode::ServerSide,
            capabilities: ToplevelCapabilities::default(),
            min_size: None,
            max_size: None,
        })
    }

    /// Get the Wayland surface ID
    pub fn surface_id(&self) -> u32 {
        self.surface_id
    }

    /// Get the current window state
    pub fn state(&self) -> &WaylandWindowState {
        &self.state
    }

    /// Get the current geometry
    pub fn geometry(&self) -> &WaylandGeometry {
        &self.geometry
    }

    /// Get the application ID
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Set the application ID (for desktop integration)
    pub fn set_app_id(&mut self, app_id: &str) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::set_app_id - surface: {}, app_id: '{}'",
            self.surface_id,
            app_id
        );
        self.app_id = app_id.to_string();
        // In real implementation: xdg_toplevel.set_app_id(app_id)
        Ok(())
    }

    /// Set the window title
    pub fn set_title(&mut self, title: &str) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::set_title - surface: {}, title: '{}'",
            self.surface_id,
            title
        );
        self.title = title.to_string();
        // In real implementation: xdg_toplevel.set_title(title)
        Ok(())
    }

    /// Set fullscreen state
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::set_fullscreen - surface: {}, fullscreen: {}",
            self.surface_id,
            fullscreen
        );
        if fullscreen {
            self.state.fullscreen = true;
            // In real implementation: xdg_toplevel.set_fullscreen(None)
        } else {
            self.state.fullscreen = false;
            // In real implementation: xdg_toplevel.unset_fullscreen()
        }
        Ok(())
    }

    /// Set maximized state
    pub fn set_maximized(&mut self, maximized: bool) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::set_maximized - surface: {}, maximized: {}",
            self.surface_id,
            maximized
        );
        if maximized {
            self.state.maximized = true;
            // In real implementation: xdg_toplevel.set_maximized()
        } else {
            self.state.maximized = false;
            // In real implementation: xdg_toplevel.unset_maximized()
        }
        Ok(())
    }

    /// Set minimum window size
    pub fn set_min_size(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::set_min_size - surface: {}, min: {}x{}",
            self.surface_id,
            width,
            height
        );
        self.min_size = Some((width, height));
        // In real implementation: xdg_toplevel.set_min_size(width, height)
        Ok(())
    }

    /// Set maximum window size
    pub fn set_max_size(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::set_max_size - surface: {}, max: {}x{}",
            self.surface_id,
            width,
            height
        );
        self.max_size = Some((width, height));
        // In real implementation: xdg_toplevel.set_max_size(width, height)
        Ok(())
    }

    /// Acknowledge a configure event
    pub fn ack_configure(&mut self, serial: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::ack_configure - surface: {}, serial: {}",
            self.surface_id,
            serial
        );
        self.pending_configure_serial = None;
        // In real implementation: xdg_surface.ack_configure(serial)
        Ok(())
    }

    /// Commit the surface (make changes visible)
    pub fn commit(&self) -> Result<(), WindowError> {
        log::trace!("LinuxWaylandWindow::commit - surface: {}", self.surface_id);
        // In real implementation: wl_surface.commit()
        Ok(())
    }

    /// Request the compositor to initiate an interactive move
    pub fn start_interactive_move(&self, serial: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::start_interactive_move - surface: {}, serial: {}",
            self.surface_id,
            serial
        );
        // In real implementation: xdg_toplevel.move_(seat, serial)
        Ok(())
    }

    /// Request the compositor to initiate an interactive resize
    pub fn start_interactive_resize(&self, serial: u32, edges: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::start_interactive_resize - surface: {}, serial: {}, edges: {}",
            self.surface_id,
            serial,
            edges
        );
        // In real implementation: xdg_toplevel.resize(seat, serial, edges)
        Ok(())
    }

    /// Set the parent window (for transient windows)
    pub fn set_parent(&self, parent_surface_id: Option<u32>) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::set_parent - surface: {}, parent: {:?}",
            self.surface_id,
            parent_surface_id
        );
        // In real implementation: xdg_toplevel.set_parent(parent_xdg_toplevel)
        Ok(())
    }

    /// Request to close the window (sends close event)
    pub fn close(&self) -> Result<(), WindowError> {
        log::debug!("LinuxWaylandWindow::close - surface: {}", self.surface_id);
        // This would typically be handled by receiving close event from compositor
        Ok(())
    }

    /// Handle a configure event from the compositor
    pub fn handle_configure(&mut self, config: WaylandSurfaceConfig) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::handle_configure - surface: {}, serial: {}, size: {}x{}",
            self.surface_id,
            config.serial,
            config.width,
            config.height
        );

        // Update geometry if compositor suggests a size
        if config.width > 0 {
            self.geometry.width = config.width;
        }
        if config.height > 0 {
            self.geometry.height = config.height;
        }

        // Update state from configure states
        for state in &config.states {
            match state {
                WaylandConfigureState::Maximized => self.state.maximized = true,
                WaylandConfigureState::Fullscreen => self.state.fullscreen = true,
                WaylandConfigureState::Resizing => self.state.resizing = true,
                WaylandConfigureState::Activated => self.state.activated = true,
                WaylandConfigureState::TiledLeft => self.state.tiled.left = true,
                WaylandConfigureState::TiledRight => self.state.tiled.right = true,
                WaylandConfigureState::TiledTop => self.state.tiled.top = true,
                WaylandConfigureState::TiledBottom => self.state.tiled.bottom = true,
            }
        }

        self.pending_configure_serial = Some(config.serial);
        Ok(())
    }
}

impl PlatformWindow for LinuxWaylandWindow {
    fn create(config: &WindowConfig) -> Result<Self, WindowError> {
        Self::create_with_app_id(config, "com.corten.browser")
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWaylandWindow::destroy - surface: {}", self.surface_id);
        // In real implementation: destroy xdg_toplevel, xdg_surface, wl_surface
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWaylandWindow::show - surface: {}", self.surface_id);
        self.state.mapped = true;
        // In Wayland, showing is done by committing a buffer to the surface
        self.commit()
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWaylandWindow::hide - surface: {}", self.surface_id);
        self.state.mapped = false;
        // In Wayland, hiding is done by attaching NULL buffer and committing
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::resize - surface: {}, size: {}x{}",
            self.surface_id,
            width,
            height
        );
        self.geometry.width = width;
        self.geometry.height = height;
        // Note: In Wayland, actual resize happens through configure events
        // This just requests the size change
        Ok(())
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), WindowError> {
        log::debug!(
            "LinuxWaylandWindow::move_to - surface: {}, position: ({}, {})",
            self.surface_id,
            x,
            y
        );
        // Note: In Wayland, windows cannot be positioned by the client
        // This is a no-op or could return an error
        log::warn!("Wayland windows cannot be positioned by client - move_to is a no-op");
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        log::debug!("LinuxWaylandWindow::focus - surface: {}", self.surface_id);
        // Note: In Wayland, windows cannot request focus
        // Focus is controlled entirely by the compositor
        log::warn!("Wayland windows cannot request focus - focus is a no-op");
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        PlatformHandle::LinuxWayland(LinuxWaylandHandle {
            surface_id: self.surface_id,
            display: self.display,
            xdg_toplevel: self.xdg_toplevel,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlatformWindow;

    #[test]
    fn test_wayland_window_create() {
        let config = WindowConfig::default();
        let result = LinuxWaylandWindow::create(&config);
        assert!(result.is_ok());
        let window = result.unwrap();
        assert!(window.surface_id() > 0);
    }

    #[test]
    fn test_wayland_window_operations() {
        let config = WindowConfig::default();
        let mut window = LinuxWaylandWindow::create(&config).unwrap();

        assert!(window.show().is_ok());
        assert!(window.state().mapped);

        assert!(window.hide().is_ok());
        assert!(!window.state().mapped);

        assert!(window.resize(800, 600).is_ok());
        assert_eq!(window.geometry().width, 800);
        assert_eq!(window.geometry().height, 600);

        assert!(window.destroy().is_ok());
    }

    #[test]
    fn test_wayland_window_handle() {
        let config = WindowConfig::default();
        let window = LinuxWaylandWindow::create(&config).unwrap();
        let handle = window.get_handle();

        assert!(handle.is_wayland());
        match handle {
            PlatformHandle::LinuxWayland(h) => {
                assert_eq!(h.surface_id, window.surface_id());
            }
            _ => panic!("Expected Wayland handle"),
        }
    }

    #[test]
    fn test_wayland_window_fullscreen() {
        let config = WindowConfig::default();
        let mut window = LinuxWaylandWindow::create(&config).unwrap();

        assert!(!window.state().fullscreen);
        assert!(window.set_fullscreen(true).is_ok());
        assert!(window.state().fullscreen);
        assert!(window.set_fullscreen(false).is_ok());
        assert!(!window.state().fullscreen);
    }

    #[test]
    fn test_wayland_window_app_id() {
        let config = WindowConfig::default();
        let mut window = LinuxWaylandWindow::create_with_app_id(&config, "test.app").unwrap();

        assert_eq!(window.app_id(), "test.app");
        assert!(window.set_app_id("new.app.id").is_ok());
        assert_eq!(window.app_id(), "new.app.id");
    }

    #[test]
    fn test_wayland_configure_handling() {
        let config = WindowConfig::default();
        let mut window = LinuxWaylandWindow::create(&config).unwrap();

        let configure = WaylandSurfaceConfig {
            width: 1024,
            height: 768,
            serial: 42,
            states: vec![WaylandConfigureState::Activated, WaylandConfigureState::Maximized],
        };

        assert!(window.handle_configure(configure).is_ok());
        assert_eq!(window.geometry().width, 1024);
        assert_eq!(window.geometry().height, 768);
        assert!(window.state().activated);
        assert!(window.state().maximized);

        // Ack the configure
        assert!(window.ack_configure(42).is_ok());
    }

    #[test]
    fn test_wayland_size_constraints() {
        let config = WindowConfig::default();
        let mut window = LinuxWaylandWindow::create(&config).unwrap();

        assert!(window.set_min_size(320, 240).is_ok());
        assert_eq!(window.min_size, Some((320, 240)));

        assert!(window.set_max_size(1920, 1080).is_ok());
        assert_eq!(window.max_size, Some((1920, 1080)));
    }
}
