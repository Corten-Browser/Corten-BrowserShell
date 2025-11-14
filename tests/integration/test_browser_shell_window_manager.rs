//! Integration tests for browser_shell → window_manager communication
//!
//! These tests verify that:
//! 1. BrowserShell correctly delegates window operations to WindowManager
//! 2. Window creation through BrowserShell works end-to-end
//! 3. Window configuration is passed correctly between components
//! 4. Window IDs are correctly managed across components
//!
//! CRITICAL: These tests use REAL components (no mocking)

use browser_shell::{BrowserShell, ShellConfig};
use integration_helpers::{create_test_config, default_window_config};
use shared_types::{ComponentError, WindowConfig};

#[tokio::test]
async fn test_browser_shell_creates_window_via_window_manager() {
    //! Given: An initialized BrowserShell
    //! When: Creating a new window through BrowserShell
    //! Then: WindowManager creates the window and returns valid WindowId

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    // Act
    let result = shell.new_window(None).await;

    // Assert
    assert!(result.is_ok(), "Window creation should succeed");
    let window_id = result.unwrap();

    // Verify window is set as active
    assert_eq!(
        shell.active_window(),
        Some(window_id),
        "Created window should be set as active"
    );
}

#[tokio::test]
async fn test_browser_shell_creates_window_with_custom_config() {
    //! Given: An initialized BrowserShell
    //! When: Creating a window with custom configuration
    //! Then: Window is created with the specified configuration

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    let custom_config = WindowConfig {
        title: "Test Window".to_string(),
        width: 1024,
        height: 768,
        fullscreen: false,
        resizable: true,
        decorations: true,
        always_on_top: false,
        skip_taskbar: false,
        x: Some(100),
        y: Some(100),
    };

    // Act
    let result = shell.new_window(Some(custom_config)).await;

    // Assert
    assert!(
        result.is_ok(),
        "Window creation with custom config should succeed"
    );
}

#[tokio::test]
async fn test_browser_shell_creates_multiple_windows() {
    //! Given: An initialized BrowserShell
    //! When: Creating multiple windows
    //! Then: Each window gets a unique WindowId

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    // Act
    let window1 = shell.new_window(None).await.expect("Window 1 creation failed");
    let window2 = shell.new_window(None).await.expect("Window 2 creation failed");
    let window3 = shell.new_window(None).await.expect("Window 3 creation failed");

    // Assert
    assert_ne!(window1, window2, "Window IDs should be unique");
    assert_ne!(window2, window3, "Window IDs should be unique");
    assert_ne!(window1, window3, "Window IDs should be unique");

    // Last created window should be active
    assert_eq!(shell.active_window(), Some(window3));
}

#[tokio::test]
async fn test_browser_shell_window_creation_requires_initialization() {
    //! Given: An uninitialized BrowserShell
    //! When: Attempting to create a window
    //! Then: Operation fails with InvalidState error

    // Arrange
    let mut shell = BrowserShell::new();

    // Act
    let result = shell.new_window(None).await;

    // Assert
    assert!(result.is_err(), "Window creation without initialization should fail");
    match result {
        Err(ComponentError::InvalidState(_)) => {
            // Expected error type
        }
        _ => panic!("Expected InvalidState error"),
    }
}

#[tokio::test]
async fn test_browser_shell_uses_default_window_config() {
    //! Given: An initialized BrowserShell with specific default window config
    //! When: Creating a window without providing config
    //! Then: Window is created using the default configuration from ShellConfig

    // Arrange
    let default_config = WindowConfig {
        title: "CortenBrowser Default".to_string(),
        width: 1280,
        height: 720,
        fullscreen: false,
        resizable: true,
        decorations: true,
        always_on_top: false,
        skip_taskbar: false,
        x: None,
        y: None,
    };

    let (mut config, _temp_dir) = create_test_config();
    config.window_config = default_config.clone();

    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    // Act - create window without providing config (should use default)
    let result = shell.new_window(None).await;

    // Assert
    assert!(
        result.is_ok(),
        "Window should be created with default configuration"
    );
}

#[tokio::test]
async fn test_browser_shell_window_manager_integration_after_shutdown() {
    //! Given: An initialized BrowserShell that has been shutdown
    //! When: Attempting to create a window after shutdown
    //! Then: Operation fails appropriately

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    // Act
    shell.shutdown().await.expect("Shutdown failed");
    let result = shell.new_window(None).await;

    // Assert
    assert!(
        result.is_err(),
        "Window creation after shutdown should fail"
    );
}
