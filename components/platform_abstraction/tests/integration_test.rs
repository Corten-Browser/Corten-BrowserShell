//! Integration tests for platform abstraction
//!
//! These tests verify cross-platform compatibility and platform detection.

use platform_abstraction::{
    create_platform_window, create_window_for_display_server, current_platform,
    detect_display_server, DisplayServer, LinuxX11Window, MacWindow, Platform, PlatformHandle,
    PlatformWindow, WindowsWindow,
};
use shared_types::WindowConfig;

/// Test that platform detection works correctly
#[test]
fn test_platform_detection() {
    let platform = current_platform();
    let display_server = detect_display_server();

    // Should return valid values
    assert!(matches!(
        platform,
        Platform::Linux | Platform::Windows | Platform::MacOS | Platform::Unknown
    ));
    assert!(matches!(
        display_server,
        DisplayServer::X11
            | DisplayServer::Wayland
            | DisplayServer::XWayland
            | DisplayServer::WindowsDwm
            | DisplayServer::MacOSQuartz
            | DisplayServer::Headless
            | DisplayServer::Unknown
    ));

    println!(
        "Detected platform: {:?}, display server: {:?}",
        platform, display_server
    );
}

/// Test creating windows with auto-detection
#[test]
fn test_auto_platform_window() {
    let config = WindowConfig::default();
    let window = create_platform_window(&config);
    assert!(window.is_ok(), "Should create a window for any platform");

    let window = window.unwrap();
    let handle = window.get_handle();

    // Handle should be valid for detected platform
    assert!(handle.raw_id() > 0, "Handle should have a non-zero ID");
}

/// Test that Linux X11 window handles are correct
#[test]
fn test_linux_x11_platform() {
    let config = WindowConfig::default();
    let window = LinuxX11Window::create(&config).expect("Failed to create X11 window");
    match window.get_handle() {
        PlatformHandle::LinuxX11(h) => {
            assert!(h.window > 0, "X11 window ID should be non-zero");
            assert_eq!(h.screen, 0, "Default screen should be 0");
        }
        _ => panic!("Expected LinuxX11 platform handle"),
    }
}

/// Test that Windows window handles are correct
#[test]
fn test_windows_platform() {
    let config = WindowConfig::default();
    let window = WindowsWindow::create(&config).expect("Failed to create Windows window");
    match window.get_handle() {
        PlatformHandle::Windows(h) => {
            assert!(h.hwnd >= 0x10000, "HWND should be above base value");
        }
        _ => panic!("Expected Windows platform handle"),
    }
}

/// Test that macOS window handles are correct
#[test]
fn test_macos_platform() {
    let config = WindowConfig::default();
    let window = MacWindow::create(&config).expect("Failed to create macOS window");
    match window.get_handle() {
        PlatformHandle::MacOS(h) => {
            assert!(h.ns_window >= 0x7FFF0000, "NSWindow should be above base value");
        }
        _ => panic!("Expected MacOS platform handle"),
    }
}

/// Test complete window lifecycle on current platform
#[test]
fn test_window_lifecycle() {
    let config = WindowConfig {
        title: "Integration Test Window".to_string(),
        width: 1280,
        height: 720,
        x: Some(50),
        y: Some(100),
        ..Default::default()
    };

    let mut window = create_platform_window(&config).expect("Failed to create window");

    // Test full lifecycle
    assert!(window.show().is_ok(), "Show failed");
    assert!(window.resize(800, 600).is_ok(), "Resize failed");
    assert!(window.move_to(200, 300).is_ok(), "Move failed");
    assert!(window.focus().is_ok(), "Focus failed");
    assert!(window.hide().is_ok(), "Hide failed");
    assert!(window.destroy().is_ok(), "Destroy failed");
}

/// Test multiple windows can be created
#[test]
fn test_multiple_windows() {
    let config1 = WindowConfig {
        title: "Window 1".to_string(),
        ..Default::default()
    };

    let config2 = WindowConfig {
        title: "Window 2".to_string(),
        width: 640,
        height: 480,
        ..Default::default()
    };

    let window1 = create_platform_window(&config1).expect("Failed to create window 1");
    let window2 = create_platform_window(&config2).expect("Failed to create window 2");

    let handle1 = window1.get_handle();
    let handle2 = window2.get_handle();

    // Handles should be different
    assert_ne!(
        handle1.raw_id(),
        handle2.raw_id(),
        "Window handles should be unique"
    );
}

/// Test creating windows for specific display servers
#[test]
fn test_explicit_display_server_windows() {
    let config = WindowConfig::default();

    // Test X11
    let x11_window = create_window_for_display_server(&config, DisplayServer::X11);
    assert!(x11_window.is_ok());
    assert!(x11_window.unwrap().get_handle().is_x11());

    // Test Wayland
    let wayland_window = create_window_for_display_server(&config, DisplayServer::Wayland);
    assert!(wayland_window.is_ok());
    assert!(wayland_window.unwrap().get_handle().is_wayland());

    // Test Windows DWM
    let win_window = create_window_for_display_server(&config, DisplayServer::WindowsDwm);
    assert!(win_window.is_ok());
    assert!(win_window.unwrap().get_handle().is_windows());

    // Test macOS Quartz
    let mac_window = create_window_for_display_server(&config, DisplayServer::MacOSQuartz);
    assert!(mac_window.is_ok());
    assert!(mac_window.unwrap().get_handle().is_macos());

    // Test Headless
    let stub_window = create_window_for_display_server(&config, DisplayServer::Headless);
    assert!(stub_window.is_ok());
    assert!(stub_window.unwrap().get_handle().is_stub());
}

/// Test window operations are idempotent
#[test]
fn test_operations_idempotent() {
    let config = WindowConfig::default();
    let mut window = create_platform_window(&config).expect("Failed to create window");

    // Multiple calls should not fail
    assert!(window.show().is_ok());
    assert!(window.show().is_ok());
    assert!(window.show().is_ok());

    assert!(window.hide().is_ok());
    assert!(window.hide().is_ok());

    assert!(window.focus().is_ok());
    assert!(window.focus().is_ok());
}

/// Test edge case: very small window dimensions
#[test]
fn test_minimal_dimensions() {
    let config = WindowConfig {
        title: "Tiny Window".to_string(),
        width: 1,
        height: 1,
        ..Default::default()
    };

    let result = create_platform_window(&config);
    assert!(result.is_ok(), "Should handle minimal dimensions");
}

/// Test edge case: very large window dimensions
#[test]
fn test_maximal_dimensions() {
    let config = WindowConfig {
        title: "Huge Window".to_string(),
        width: u32::MAX / 2,
        height: u32::MAX / 2,
        ..Default::default()
    };

    let result = create_platform_window(&config);
    assert!(result.is_ok(), "Should handle maximal dimensions");
}

/// Test window with all config options
#[test]
fn test_full_config() {
    let config = WindowConfig {
        title: "Fully Configured Window".to_string(),
        width: 1920,
        height: 1080,
        x: Some(-100), // Negative positions should work
        y: Some(-50),
        fullscreen: true,
        resizable: false,
        decorations: false,
        always_on_top: true,
        skip_taskbar: true,
    };

    let result = create_platform_window(&config);
    assert!(result.is_ok(), "Should handle full config");
}

/// Test platform handle utilities
#[test]
fn test_platform_handle_utilities() {
    let config = WindowConfig::default();

    // X11
    let x11 = create_window_for_display_server(&config, DisplayServer::X11).unwrap();
    let handle = x11.get_handle();
    assert!(handle.is_x11());
    assert!(!handle.is_wayland());
    assert!(!handle.is_windows());
    assert!(!handle.is_macos());
    assert!(!handle.is_stub());

    // Wayland
    let wayland = create_window_for_display_server(&config, DisplayServer::Wayland).unwrap();
    let handle = wayland.get_handle();
    assert!(handle.is_wayland());
    assert!(!handle.is_x11());

    // Windows
    let win = create_window_for_display_server(&config, DisplayServer::WindowsDwm).unwrap();
    let handle = win.get_handle();
    assert!(handle.is_windows());

    // macOS
    let mac = create_window_for_display_server(&config, DisplayServer::MacOSQuartz).unwrap();
    let handle = mac.get_handle();
    assert!(handle.is_macos());
}
