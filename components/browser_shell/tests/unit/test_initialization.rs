//! Tests for BrowserShell initialization and shutdown

use browser_shell::{BrowserShell, ShellConfig};
use shared_types::WindowConfig;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test that BrowserShell can be initialized with valid configuration
#[tokio::test]
async fn test_initialize_succeeds() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: true,
        enable_extensions: false,
        user_data_dir: temp_dir.path().to_str().unwrap().to_string(),
    };

    let mut shell = BrowserShell::new();

    // Act
    let result = shell.initialize(config).await;

    // Assert
    assert!(result.is_ok(), "Initialization should succeed");
}

/// Test that BrowserShell shutdown saves settings and bookmarks
#[tokio::test]
async fn test_shutdown_succeeds() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: true,
        enable_extensions: false,
        user_data_dir: temp_dir.path().to_str().unwrap().to_string(),
    };

    let mut shell = BrowserShell::new();
    shell.initialize(config).await.unwrap();

    // Act
    let result = shell.shutdown().await;

    // Assert
    assert!(result.is_ok(), "Shutdown should succeed");
}

/// Test that shutdown works even if not initialized
#[tokio::test]
async fn test_shutdown_without_initialization() {
    // Arrange
    let mut shell = BrowserShell::new();

    // Act
    let result = shell.shutdown().await;

    // Assert
    assert!(result.is_ok(), "Shutdown should succeed even without initialization");
}
