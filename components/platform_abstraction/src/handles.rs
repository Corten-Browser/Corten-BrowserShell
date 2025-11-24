//! Platform-specific handle types
//!
//! These types represent platform-specific window handles that can be used
//! for interoperability with native windowing systems.

use serde::{Deserialize, Serialize};

/// Platform-specific window handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformHandle {
    /// Linux X11 window handle
    LinuxX11(LinuxX11Handle),
    /// Linux Wayland window handle
    LinuxWayland(LinuxWaylandHandle),
    /// Windows HWND handle
    Windows(WindowsHandle),
    /// macOS NSWindow handle
    MacOS(MacOSHandle),
    /// Stub/Mock handle for testing
    Stub(StubHandle),
}

/// Linux X11 window handle (for x11rb integration)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinuxX11Handle {
    /// X11 Window ID (XID)
    pub window: u32,
    /// X11 Display connection identifier
    pub display: usize,
    /// Screen number
    pub screen: i32,
    /// Visual ID for the window
    pub visual_id: u32,
}

/// Linux Wayland window handle (for wayland-client integration)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinuxWaylandHandle {
    /// Wayland surface ID
    pub surface_id: u32,
    /// Wayland display pointer
    pub display: usize,
    /// XDG toplevel handle (for xdg-shell)
    pub xdg_toplevel: usize,
}

/// Deprecated: Use LinuxX11Handle or LinuxWaylandHandle instead
/// Kept for backwards compatibility
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinuxHandle {
    /// X11/Wayland window ID
    pub window: u32,
}

impl From<LinuxX11Handle> for LinuxHandle {
    fn from(handle: LinuxX11Handle) -> Self {
        LinuxHandle {
            window: handle.window,
        }
    }
}

impl From<LinuxWaylandHandle> for LinuxHandle {
    fn from(handle: LinuxWaylandHandle) -> Self {
        LinuxHandle {
            window: handle.surface_id,
        }
    }
}

/// Windows window handle (for windows-rs integration)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowsHandle {
    /// HWND as raw pointer value
    pub hwnd: usize,
    /// HINSTANCE of the application
    pub hinstance: usize,
    /// Window class name hash (for identification)
    pub class_name_hash: u32,
}

/// macOS window handle (for cocoa integration)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MacOSHandle {
    /// NSWindow pointer value
    pub ns_window: usize,
    /// NSView pointer value for the content view
    pub ns_view: usize,
    /// CALayer pointer for Core Animation
    pub ca_layer: usize,
}

/// Stub handle for testing/mock implementations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StubHandle {
    /// Mock window ID
    pub id: u64,
}

impl PlatformHandle {
    /// Check if this is an X11 handle
    pub fn is_x11(&self) -> bool {
        matches!(self, PlatformHandle::LinuxX11(_))
    }

    /// Check if this is a Wayland handle
    pub fn is_wayland(&self) -> bool {
        matches!(self, PlatformHandle::LinuxWayland(_))
    }

    /// Check if this is a Windows handle
    pub fn is_windows(&self) -> bool {
        matches!(self, PlatformHandle::Windows(_))
    }

    /// Check if this is a macOS handle
    pub fn is_macos(&self) -> bool {
        matches!(self, PlatformHandle::MacOS(_))
    }

    /// Check if this is a stub/mock handle
    pub fn is_stub(&self) -> bool {
        matches!(self, PlatformHandle::Stub(_))
    }

    /// Get the raw window ID as a u64 (platform-independent)
    pub fn raw_id(&self) -> u64 {
        match self {
            PlatformHandle::LinuxX11(h) => h.window as u64,
            PlatformHandle::LinuxWayland(h) => h.surface_id as u64,
            PlatformHandle::Windows(h) => h.hwnd as u64,
            PlatformHandle::MacOS(h) => h.ns_window as u64,
            PlatformHandle::Stub(h) => h.id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_x11_handle_creation() {
        let handle = LinuxX11Handle {
            window: 12345,
            display: 0x7fff0000,
            screen: 0,
            visual_id: 0x21,
        };
        assert_eq!(handle.window, 12345);
        assert_eq!(handle.screen, 0);
    }

    #[test]
    fn test_linux_wayland_handle_creation() {
        let handle = LinuxWaylandHandle {
            surface_id: 54321,
            display: 0x7fff0000,
            xdg_toplevel: 0x7fff1000,
        };
        assert_eq!(handle.surface_id, 54321);
    }

    #[test]
    fn test_windows_handle_creation() {
        let handle = WindowsHandle {
            hwnd: 0x1234ABCD,
            hinstance: 0x00400000,
            class_name_hash: 0xDEADBEEF,
        };
        assert_eq!(handle.hwnd, 0x1234ABCD);
    }

    #[test]
    fn test_macos_handle_creation() {
        let handle = MacOSHandle {
            ns_window: 0xDEADBEEF,
            ns_view: 0xCAFEBABE,
            ca_layer: 0xFEEDFACE,
        };
        assert_eq!(handle.ns_window, 0xDEADBEEF);
    }

    #[test]
    fn test_platform_handle_variants() {
        let x11 = PlatformHandle::LinuxX11(LinuxX11Handle {
            window: 1,
            display: 0,
            screen: 0,
            visual_id: 0,
        });
        let wayland = PlatformHandle::LinuxWayland(LinuxWaylandHandle {
            surface_id: 2,
            display: 0,
            xdg_toplevel: 0,
        });
        let windows = PlatformHandle::Windows(WindowsHandle {
            hwnd: 3,
            hinstance: 0,
            class_name_hash: 0,
        });
        let macos = PlatformHandle::MacOS(MacOSHandle {
            ns_window: 4,
            ns_view: 0,
            ca_layer: 0,
        });

        assert!(x11.is_x11());
        assert!(wayland.is_wayland());
        assert!(windows.is_windows());
        assert!(macos.is_macos());
    }

    #[test]
    fn test_raw_id_extraction() {
        let x11 = PlatformHandle::LinuxX11(LinuxX11Handle {
            window: 100,
            display: 0,
            screen: 0,
            visual_id: 0,
        });
        assert_eq!(x11.raw_id(), 100);

        let windows = PlatformHandle::Windows(WindowsHandle {
            hwnd: 200,
            hinstance: 0,
            class_name_hash: 0,
        });
        assert_eq!(windows.raw_id(), 200);
    }

    #[test]
    fn test_linux_handle_conversion() {
        let x11 = LinuxX11Handle {
            window: 999,
            display: 0,
            screen: 0,
            visual_id: 0,
        };
        let legacy: LinuxHandle = x11.into();
        assert_eq!(legacy.window, 999);

        let wayland = LinuxWaylandHandle {
            surface_id: 888,
            display: 0,
            xdg_toplevel: 0,
        };
        let legacy2: LinuxHandle = wayland.into();
        assert_eq!(legacy2.window, 888);
    }
}
