//! Integration tests for platform abstraction
//!
//! These tests verify cross-platform compatibility and platform detection.

use platform_abstraction::{PlatformHandle, PlatformWindow};
use shared_types::WindowConfig;

#[cfg(target_os = "linux")]
use platform_abstraction::LinuxWindow;

#[cfg(target_os = "windows")]
use platform_abstraction::WindowsWindow;

#[cfg(target_os = "macos")]
use platform_abstraction::MacWindow;

/// Test that platform detection works correctly
#[test]
fn test_platform_detection() {
    #[cfg(target_os = "linux")]
    {
        let config = WindowConfig::default();
        let window = LinuxWindow::create(&config).expect("Failed to create Linux window");
        match window.get_handle() {
            PlatformHandle::Linux(_) => {} // Correct
            _ => panic!("Expected Linux platform handle"),
        }
    }

    #[cfg(target_os = "windows")]
    {
        let config = WindowConfig::default();
        let window = WindowsWindow::create(&config).expect("Failed to create Windows window");
        match window.get_handle() {
            PlatformHandle::Windows(_) => {} // Correct
            _ => panic!("Expected Windows platform handle"),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let config = WindowConfig::default();
        let window = MacWindow::create(&config).expect("Failed to create macOS window");
        match window.get_handle() {
            PlatformHandle::MacOS(_) => {} // Correct
            _ => panic!("Expected MacOS platform handle"),
        }
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

    #[cfg(target_os = "linux")]
    let mut window = LinuxWindow::create(&config).expect("Failed to create window");

    #[cfg(target_os = "windows")]
    let mut window = WindowsWindow::create(&config).expect("Failed to create window");

    #[cfg(target_os = "macos")]
    let mut window = MacWindow::create(&config).expect("Failed to create window");

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

    #[cfg(target_os = "linux")]
    {
        let window1 = LinuxWindow::create(&config1).expect("Failed to create window 1");
        let window2 = LinuxWindow::create(&config2).expect("Failed to create window 2");

        let handle1 = window1.get_handle();
        let handle2 = window2.get_handle();

        // Handles should be different
        match (handle1, handle2) {
            (PlatformHandle::Linux(h1), PlatformHandle::Linux(h2)) => {
                assert_ne!(h1.window, h2.window, "Window handles should be unique");
            }
            _ => panic!("Unexpected handle types"),
        }
    }

    #[cfg(target_os = "windows")]
    {
        let window1 = WindowsWindow::create(&config1).expect("Failed to create window 1");
        let window2 = WindowsWindow::create(&config2).expect("Failed to create window 2");

        let handle1 = window1.get_handle();
        let handle2 = window2.get_handle();

        // Handles should be different
        match (handle1, handle2) {
            (PlatformHandle::Windows(h1), PlatformHandle::Windows(h2)) => {
                assert_ne!(h1.hwnd, h2.hwnd, "Window handles should be unique");
            }
            _ => panic!("Unexpected handle types"),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let window1 = MacWindow::create(&config1).expect("Failed to create window 1");
        let window2 = MacWindow::create(&config2).expect("Failed to create window 2");

        let handle1 = window1.get_handle();
        let handle2 = window2.get_handle();

        // Handles should be different
        match (handle1, handle2) {
            (PlatformHandle::MacOS(h1), PlatformHandle::MacOS(h2)) => {
                assert_ne!(
                    h1.ns_window, h2.ns_window,
                    "Window handles should be unique"
                );
            }
            _ => panic!("Unexpected handle types"),
        }
    }
}

/// Test window operations are idempotent
#[test]
fn test_operations_idempotent() {
    let config = WindowConfig::default();

    #[cfg(target_os = "linux")]
    let mut window = LinuxWindow::create(&config).expect("Failed to create window");

    #[cfg(target_os = "windows")]
    let mut window = WindowsWindow::create(&config).expect("Failed to create window");

    #[cfg(target_os = "macos")]
    let mut window = MacWindow::create(&config).expect("Failed to create window");

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

    #[cfg(target_os = "linux")]
    let result = LinuxWindow::create(&config);

    #[cfg(target_os = "windows")]
    let result = WindowsWindow::create(&config);

    #[cfg(target_os = "macos")]
    let result = MacWindow::create(&config);

    // Stub implementation should accept any dimensions
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

    #[cfg(target_os = "linux")]
    let result = LinuxWindow::create(&config);

    #[cfg(target_os = "windows")]
    let result = WindowsWindow::create(&config);

    #[cfg(target_os = "macos")]
    let result = MacWindow::create(&config);

    // Stub implementation should accept any dimensions
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

    #[cfg(target_os = "linux")]
    let result = LinuxWindow::create(&config);

    #[cfg(target_os = "windows")]
    let result = WindowsWindow::create(&config);

    #[cfg(target_os = "macos")]
    let result = MacWindow::create(&config);

    assert!(result.is_ok(), "Should handle full config");
}
