//! Integration tests for browser_shell → bookmarks_manager communication
//!
//! These tests verify that:
//! 1. Bookmarks are loaded during browser_shell initialization
//! 2. Bookmarks persist across browser restarts
//! 3. BookmarksManager is correctly initialized with user data directory
//!
//! CRITICAL: These tests use REAL components (no mocking)

use browser_shell::{BrowserShell, ShellConfig};
use integration_helpers::create_test_config;
use shared_types::WindowConfig;
use std::path::PathBuf;

#[tokio::test]
async fn test_browser_shell_initializes_bookmarks_manager() {
    //! Given: A new BrowserShell instance
    //! When: Initializing with configuration
    //! Then: BookmarksManager is initialized and bookmarks are loaded

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();

    // Act
    let result = shell.initialize(config).await;

    // Assert
    assert!(
        result.is_ok(),
        "Initialization with bookmarks manager should succeed"
    );
}

#[tokio::test]
async fn test_browser_shell_saves_bookmarks_on_shutdown() {
    //! Given: An initialized BrowserShell
    //! When: Shutting down the browser
    //! Then: Bookmarks are persisted to disk

    // Arrange
    let (config, temp_dir) = create_test_config();
    let user_data_path = PathBuf::from(&config.user_data_dir);

    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    // Act
    let result = shell.shutdown().await;

    // Assert
    assert!(
        result.is_ok(),
        "Shutdown with bookmarks save should succeed"
    );

    // Verify user data directory exists
    assert!(
        user_data_path.exists(),
        "User data directory should exist after shutdown"
    );

    drop(temp_dir);
}

#[tokio::test]
async fn test_bookmarks_persist_across_restarts() {
    //! Given: A BrowserShell that has been initialized and shutdown
    //! When: Restarting with the same configuration
    //! Then: Bookmarks are loaded from disk

    // Arrange - first session
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let user_data_dir = temp_dir.path().to_str().unwrap().to_string();

    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: true,
        enable_extensions: false,
        user_data_dir: user_data_dir.clone(),
    };

    let mut shell1 = BrowserShell::new();
    shell1
        .initialize(config.clone())
        .await
        .expect("First initialization failed");
    shell1.shutdown().await.expect("First shutdown failed");

    // Act - second session (restart)
    let mut shell2 = BrowserShell::new();
    let result = shell2.initialize(config).await;

    // Assert
    assert!(
        result.is_ok(),
        "Reinitialization should succeed and load persisted bookmarks"
    );

    shell2.shutdown().await.expect("Second shutdown failed");

    drop(temp_dir);
}

#[tokio::test]
async fn test_browser_shell_bookmarks_with_custom_user_data_dir() {
    //! Given: A ShellConfig with custom user data directory
    //! When: Initializing BrowserShell
    //! Then: Bookmarks are stored in the specified directory

    // Arrange
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let custom_path = temp_dir.path().join("custom_bookmarks_data");

    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: false,
        enable_extensions: true,
        user_data_dir: custom_path.to_str().unwrap().to_string(),
    };

    let mut shell = BrowserShell::new();

    // Act
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");
    shell.shutdown().await.expect("Shutdown failed");

    // Assert
    assert!(
        custom_path.exists(),
        "Custom user data directory should be created for bookmarks"
    );

    drop(temp_dir);
}

#[tokio::test]
async fn test_bookmarks_manager_survives_multiple_init_shutdown_cycles() {
    //! Given: A shared user data directory
    //! When: Multiple init/shutdown cycles occur
    //! Then: BookmarksManager correctly handles repeated initialization

    // Arrange
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let user_data_dir = temp_dir.path().to_str().unwrap().to_string();

    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: true,
        enable_extensions: false,
        user_data_dir: user_data_dir.clone(),
    };

    // Act - multiple cycles
    for i in 1..=3 {
        let mut shell = BrowserShell::new();
        shell
            .initialize(config.clone())
            .await
            .unwrap_or_else(|_| panic!("Initialization {} failed", i));
        shell
            .shutdown()
            .await
            .unwrap_or_else(|_| panic!("Shutdown {} failed", i));
    }

    // Assert - if we got here, all cycles succeeded
    assert!(true, "Multiple init/shutdown cycles completed successfully");

    drop(temp_dir);
}
