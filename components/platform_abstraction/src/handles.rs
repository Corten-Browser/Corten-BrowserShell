//! Platform-specific handle types
//!
//! These types represent platform-specific window handles that can be used
//! for interoperability with native windowing systems.

use serde::{Deserialize, Serialize};

/// Platform-specific window handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformHandle {
    /// Linux/X11 window handle
    Linux(LinuxHandle),
    /// Windows HWND handle
    Windows(WindowsHandle),
    /// macOS NSWindow handle
    MacOS(MacOSHandle),
}

/// Linux/X11 window handle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinuxHandle {
    /// X11/Wayland window ID
    pub window: u32,
}

/// Windows window handle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowsHandle {
    /// HWND as raw pointer value
    pub hwnd: usize,
}

/// macOS window handle
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MacOSHandle {
    /// NSWindow pointer value
    pub ns_window: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_handle_creation() {
        let handle = LinuxHandle { window: 12345 };
        assert_eq!(handle.window, 12345);
    }

    #[test]
    fn test_windows_handle_creation() {
        let handle = WindowsHandle { hwnd: 0x1234ABCD };
        assert_eq!(handle.hwnd, 0x1234ABCD);
    }

    #[test]
    fn test_macos_handle_creation() {
        let handle = MacOSHandle {
            ns_window: 0xDEADBEEF,
        };
        assert_eq!(handle.ns_window, 0xDEADBEEF);
    }

    #[test]
    fn test_platform_handle_variants() {
        let linux = PlatformHandle::Linux(LinuxHandle { window: 1 });
        let windows = PlatformHandle::Windows(WindowsHandle { hwnd: 2 });
        let macos = PlatformHandle::MacOS(MacOSHandle { ns_window: 3 });

        match linux {
            PlatformHandle::Linux(h) => assert_eq!(h.window, 1),
            _ => panic!("Expected Linux variant"),
        }

        match windows {
            PlatformHandle::Windows(h) => assert_eq!(h.hwnd, 2),
            _ => panic!("Expected Windows variant"),
        }

        match macos {
            PlatformHandle::MacOS(h) => assert_eq!(h.ns_window, 3),
            _ => panic!("Expected MacOS variant"),
        }
    }
}
