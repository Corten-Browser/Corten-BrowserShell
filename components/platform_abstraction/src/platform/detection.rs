//! Platform detection and display server identification
//!
//! This module provides runtime detection of the current platform and
//! available display servers. On Linux, it can distinguish between
//! X11 and Wayland sessions.

use serde::{Deserialize, Serialize};

/// The current operating system platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    /// Linux (any distribution)
    Linux,
    /// Windows (any version)
    Windows,
    /// macOS (any version)
    MacOS,
    /// Unknown or unsupported platform
    Unknown,
}

/// Display server type (primarily relevant for Linux)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayServer {
    /// X11/Xorg display server
    X11,
    /// Wayland compositor
    Wayland,
    /// X11 running under XWayland (Wayland with X11 compatibility)
    XWayland,
    /// Windows Desktop Window Manager
    WindowsDwm,
    /// macOS Quartz Compositor
    MacOSQuartz,
    /// No display server (headless)
    Headless,
    /// Unknown display server
    Unknown,
}

/// Information about the current platform environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Current operating system
    pub platform: Platform,
    /// Active display server
    pub display_server: DisplayServer,
    /// Whether native window creation is supported
    pub native_supported: bool,
    /// Platform-specific details
    pub details: PlatformDetails,
}

/// Platform-specific detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDetails {
    /// Display name (e.g., ":0" for X11, "wayland-0" for Wayland)
    pub display_name: Option<String>,
    /// Screen/monitor count
    pub screen_count: u32,
    /// Primary screen index
    pub primary_screen: u32,
    /// Whether HiDPI/scaling is enabled
    pub hidpi_enabled: bool,
    /// Scale factor (1.0 = no scaling)
    pub scale_factor: f64,
}

impl Default for PlatformDetails {
    fn default() -> Self {
        Self {
            display_name: None,
            screen_count: 1,
            primary_screen: 0,
            hidpi_enabled: false,
            scale_factor: 1.0,
        }
    }
}

/// Detect the current platform at compile time
pub const fn current_platform() -> Platform {
    #[cfg(target_os = "linux")]
    {
        Platform::Linux
    }
    #[cfg(target_os = "windows")]
    {
        Platform::Windows
    }
    #[cfg(target_os = "macos")]
    {
        Platform::MacOS
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Platform::Unknown
    }
}

/// Detect the display server at runtime
pub fn detect_display_server() -> DisplayServer {
    match current_platform() {
        Platform::Linux => detect_linux_display_server(),
        Platform::Windows => DisplayServer::WindowsDwm,
        Platform::MacOS => DisplayServer::MacOSQuartz,
        Platform::Unknown => DisplayServer::Unknown,
    }
}

/// Detect Linux display server (X11 or Wayland)
fn detect_linux_display_server() -> DisplayServer {
    // Check for Wayland first (it's the modern default)
    if is_wayland_session() {
        // Check if running X11 app under XWayland
        if is_x11_available() {
            DisplayServer::XWayland
        } else {
            DisplayServer::Wayland
        }
    } else if is_x11_available() {
        DisplayServer::X11
    } else {
        // No display server (headless or SSH without X forwarding)
        DisplayServer::Headless
    }
}

/// Check if running in a Wayland session
fn is_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("XDG_SESSION_TYPE")
        .map(|v| v.to_lowercase() == "wayland")
        .unwrap_or(false)
}

/// Check if X11 is available
fn is_x11_available() -> bool {
    std::env::var("DISPLAY").is_ok()
}

/// Get comprehensive platform information
pub fn get_platform_info() -> PlatformInfo {
    let platform = current_platform();
    let display_server = detect_display_server();

    let details = match platform {
        Platform::Linux => get_linux_details(&display_server),
        Platform::Windows => get_windows_details(),
        Platform::MacOS => get_macos_details(),
        Platform::Unknown => PlatformDetails::default(),
    };

    let native_supported = match (&platform, &display_server) {
        (Platform::Linux, DisplayServer::X11) => true,
        (Platform::Linux, DisplayServer::Wayland) => true,
        (Platform::Linux, DisplayServer::XWayland) => true,
        (Platform::Windows, DisplayServer::WindowsDwm) => true,
        (Platform::MacOS, DisplayServer::MacOSQuartz) => true,
        _ => false,
    };

    PlatformInfo {
        platform,
        display_server,
        native_supported,
        details,
    }
}

fn get_linux_details(display_server: &DisplayServer) -> PlatformDetails {
    let display_name = match display_server {
        DisplayServer::X11 | DisplayServer::XWayland => std::env::var("DISPLAY").ok(),
        DisplayServer::Wayland => std::env::var("WAYLAND_DISPLAY").ok(),
        _ => None,
    };

    // Get scale factor from GDK_SCALE or QT_SCALE_FACTOR
    let scale_factor = std::env::var("GDK_SCALE")
        .or_else(|_| std::env::var("QT_SCALE_FACTOR"))
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);

    PlatformDetails {
        display_name,
        screen_count: 1, // Would need X11/Wayland connection to query
        primary_screen: 0,
        hidpi_enabled: scale_factor > 1.0,
        scale_factor,
    }
}

fn get_windows_details() -> PlatformDetails {
    // On Windows, we would use GetSystemMetrics and GetDpiForSystem
    // For stub implementation, return defaults
    PlatformDetails {
        display_name: Some("Windows Desktop".to_string()),
        screen_count: 1,
        primary_screen: 0,
        hidpi_enabled: false,
        scale_factor: 1.0,
    }
}

fn get_macos_details() -> PlatformDetails {
    // On macOS, we would use NSScreen APIs
    // For stub implementation, return defaults
    PlatformDetails {
        display_name: Some("Quartz Compositor".to_string()),
        screen_count: 1,
        primary_screen: 0,
        hidpi_enabled: true, // Retina displays are common
        scale_factor: 2.0,   // Default Retina scale
    }
}

/// Check if the current platform supports a specific display server
pub fn supports_display_server(server: DisplayServer) -> bool {
    match (current_platform(), server) {
        (Platform::Linux, DisplayServer::X11) => true,
        (Platform::Linux, DisplayServer::Wayland) => true,
        (Platform::Linux, DisplayServer::XWayland) => true,
        (Platform::Windows, DisplayServer::WindowsDwm) => true,
        (Platform::MacOS, DisplayServer::MacOSQuartz) => true,
        _ => false,
    }
}

/// Get the preferred display server for the current platform
pub fn preferred_display_server() -> DisplayServer {
    match current_platform() {
        Platform::Linux => {
            // Prefer Wayland if available, fall back to X11
            if is_wayland_session() {
                DisplayServer::Wayland
            } else {
                DisplayServer::X11
            }
        }
        Platform::Windows => DisplayServer::WindowsDwm,
        Platform::MacOS => DisplayServer::MacOSQuartz,
        Platform::Unknown => DisplayServer::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        // Should compile to one of the known platforms
        assert!(matches!(
            platform,
            Platform::Linux | Platform::Windows | Platform::MacOS | Platform::Unknown
        ));
    }

    #[test]
    fn test_detect_display_server() {
        let server = detect_display_server();
        // Should return some display server
        assert!(matches!(
            server,
            DisplayServer::X11
                | DisplayServer::Wayland
                | DisplayServer::XWayland
                | DisplayServer::WindowsDwm
                | DisplayServer::MacOSQuartz
                | DisplayServer::Headless
                | DisplayServer::Unknown
        ));
    }

    #[test]
    fn test_get_platform_info() {
        let info = get_platform_info();
        assert!(info.details.scale_factor > 0.0);
        assert!(info.details.screen_count >= 1);
    }

    #[test]
    fn test_platform_details_default() {
        let details = PlatformDetails::default();
        assert_eq!(details.scale_factor, 1.0);
        assert_eq!(details.screen_count, 1);
        assert_eq!(details.primary_screen, 0);
    }

    #[test]
    fn test_supports_display_server() {
        // Current platform should support at least one display server
        let platform = current_platform();
        let has_support = match platform {
            Platform::Linux => {
                supports_display_server(DisplayServer::X11)
                    || supports_display_server(DisplayServer::Wayland)
            }
            Platform::Windows => supports_display_server(DisplayServer::WindowsDwm),
            Platform::MacOS => supports_display_server(DisplayServer::MacOSQuartz),
            Platform::Unknown => true, // Allow unknown platforms
        };
        assert!(has_support);
    }

    #[test]
    fn test_preferred_display_server() {
        let preferred = preferred_display_server();
        // Should return a valid display server for the current platform
        assert!(!matches!(preferred, DisplayServer::Unknown)
            || matches!(current_platform(), Platform::Unknown));
    }
}
