//! Unit tests for browser shell orchestrator

use browser_shell::{BrowserShell, ShellConfig};
use shared_types::{ComponentError, WindowConfig};

#[tokio::test]
async fn test_browser_shell_initialization() {
    // Arrange
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: false,
        enable_extensions: false,
        user_data_dir: "/tmp/browser_test".to_string(),
    };

    let mut shell = BrowserShell::new();

    // Act
    let result = shell.initialize(config).await;

    // Assert
    assert!(result.is_ok(), "Browser shell initialization should succeed");
}

#[tokio::test]
async fn test_browser_shell_shutdown() {
    // Arrange
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: false,
        enable_extensions: false,
        user_data_dir: "/tmp/browser_test".to_string(),
    };

    let mut shell = BrowserShell::new();
    shell.initialize(config).await.unwrap();

    // Act
    let result = shell.shutdown().await;

    // Assert
    assert!(result.is_ok(), "Browser shell shutdown should succeed");
}

#[tokio::test]
async fn test_new_window_creation() {
    // Arrange
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: false,
        enable_extensions: false,
        user_data_dir: "/tmp/browser_test".to_string(),
    };

    let mut shell = BrowserShell::new();
    shell.initialize(config).await.unwrap();

    // Act
    let result = shell.new_window(None).await;

    // Assert
    assert!(result.is_ok(), "Creating new window should succeed");
    let window_id = result.unwrap();
    assert!(!window_id.to_string().is_empty(), "Window ID should be valid");
}

#[tokio::test]
async fn test_new_tab_creation() {
    // Arrange
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: false,
        enable_extensions: false,
        user_data_dir: "/tmp/browser_test".to_string(),
    };

    let mut shell = BrowserShell::new();
    shell.initialize(config).await.unwrap();

    // Create a window first
    shell.new_window(None).await.unwrap();

    // Act
    let result = shell.new_tab(None).await;

    // Assert
    assert!(result.is_ok(), "Creating new tab should succeed");
}

#[tokio::test]
async fn test_navigate() {
    // Arrange
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: false,
        enable_extensions: false,
        user_data_dir: "/tmp/browser_test".to_string(),
    };

    let mut shell = BrowserShell::new();
    shell.initialize(config).await.unwrap();
    shell.new_window(None).await.unwrap();
    shell.new_tab(None).await.unwrap();

    // Act
    let result = shell.navigate("https://example.com".to_string()).await;

    // Assert
    assert!(result.is_ok(), "Navigation should succeed");
}

#[tokio::test]
async fn test_new_tab_without_window_fails() {
    // Arrange
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: false,
        enable_extensions: false,
        user_data_dir: "/tmp/browser_test".to_string(),
    };

    let mut shell = BrowserShell::new();
    shell.initialize(config).await.unwrap();

    // Act
    let result = shell.new_tab(None).await;

    // Assert
    assert!(result.is_err(), "Creating tab without window should fail");
}
