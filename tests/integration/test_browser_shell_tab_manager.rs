//! Integration tests for browser_shell → tab_manager communication
//!
//! These tests verify that:
//! 1. BrowserShell correctly delegates tab operations to TabManager
//! 2. Tab creation requires an active window
//! 3. Tab navigation works through BrowserShell
//! 4. Tab IDs are correctly managed across components
//!
//! CRITICAL: These tests use REAL components (no mocking)

use browser_shell::{BrowserShell, ShellConfig};
use integration_helpers::create_test_config;
use shared_types::ComponentError;

#[tokio::test]
async fn test_browser_shell_creates_tab_via_tab_manager() {
    //! Given: An initialized BrowserShell with an active window
    //! When: Creating a new tab through BrowserShell
    //! Then: TabManager creates the tab and returns valid TabId

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    // Create window first (required for tab creation)
    shell
        .new_window(None)
        .await
        .expect("Window creation failed");

    // Act
    let result = shell.new_tab(None).await;

    // Assert
    assert!(result.is_ok(), "Tab creation should succeed");
    let tab_id = result.unwrap();

    // Verify tab is set as active
    assert_eq!(
        shell.active_tab(),
        Some(tab_id),
        "Created tab should be set as active"
    );
}

#[tokio::test]
async fn test_browser_shell_creates_tab_with_url() {
    //! Given: An initialized BrowserShell with an active window
    //! When: Creating a tab with a specific URL
    //! Then: Tab is created and will navigate to the specified URL

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    shell
        .new_window(None)
        .await
        .expect("Window creation failed");

    let test_url = "https://example.com";

    // Act
    let result = shell.new_tab(Some(test_url.to_string())).await;

    // Assert
    assert!(result.is_ok(), "Tab creation with URL should succeed");
}

#[tokio::test]
async fn test_browser_shell_tab_creation_requires_active_window() {
    //! Given: An initialized BrowserShell without any windows
    //! When: Attempting to create a tab
    //! Then: Operation fails with InvalidState error

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    // Act - try to create tab without creating window first
    let result = shell.new_tab(None).await;

    // Assert
    assert!(
        result.is_err(),
        "Tab creation without active window should fail"
    );
    match result {
        Err(ComponentError::InvalidState(msg)) => {
            assert!(
                msg.contains("No active window"),
                "Error should indicate missing active window"
            );
        }
        _ => panic!("Expected InvalidState error about missing active window"),
    }
}

#[tokio::test]
async fn test_browser_shell_creates_multiple_tabs() {
    //! Given: An initialized BrowserShell with an active window
    //! When: Creating multiple tabs
    //! Then: Each tab gets a unique TabId

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    shell
        .new_window(None)
        .await
        .expect("Window creation failed");

    // Act
    let tab1 = shell.new_tab(None).await.expect("Tab 1 creation failed");
    let tab2 = shell.new_tab(None).await.expect("Tab 2 creation failed");
    let tab3 = shell.new_tab(None).await.expect("Tab 3 creation failed");

    // Assert
    assert_ne!(tab1, tab2, "Tab IDs should be unique");
    assert_ne!(tab2, tab3, "Tab IDs should be unique");
    assert_ne!(tab1, tab3, "Tab IDs should be unique");

    // Last created tab should be active
    assert_eq!(shell.active_tab(), Some(tab3));
}

#[tokio::test]
async fn test_browser_shell_navigate_active_tab() {
    //! Given: An initialized BrowserShell with a window and tab
    //! When: Navigating the active tab to a URL
    //! Then: TabManager navigates the tab successfully

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    shell
        .new_window(None)
        .await
        .expect("Window creation failed");
    shell.new_tab(None).await.expect("Tab creation failed");

    let test_url = "https://rust-lang.org";

    // Act
    let result = shell.navigate(test_url.to_string()).await;

    // Assert
    assert!(result.is_ok(), "Navigation should succeed");
}

#[tokio::test]
async fn test_browser_shell_navigate_requires_active_tab() {
    //! Given: An initialized BrowserShell without any tabs
    //! When: Attempting to navigate
    //! Then: Operation fails with InvalidState error

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    // Create window but no tab
    shell
        .new_window(None)
        .await
        .expect("Window creation failed");

    // Act
    let result = shell.navigate("https://example.com".to_string()).await;

    // Assert
    assert!(result.is_err(), "Navigation without active tab should fail");
    match result {
        Err(ComponentError::InvalidState(msg)) => {
            assert!(
                msg.contains("No active tab"),
                "Error should indicate missing active tab"
            );
        }
        _ => panic!("Expected InvalidState error about missing active tab"),
    }
}

#[tokio::test]
async fn test_browser_shell_tab_navigation_flow() {
    //! Given: An initialized BrowserShell
    //! When: Creating window, tab, and navigating in sequence
    //! Then: Complete flow works end-to-end

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    // Act & Assert - complete flow
    let window_id = shell
        .new_window(None)
        .await
        .expect("Window creation failed");
    assert_eq!(shell.active_window(), Some(window_id));

    let tab_id = shell.new_tab(None).await.expect("Tab creation failed");
    assert_eq!(shell.active_tab(), Some(tab_id));

    shell
        .navigate("https://www.rust-lang.org".to_string())
        .await
        .expect("First navigation failed");

    shell
        .navigate("https://doc.rust-lang.org".to_string())
        .await
        .expect("Second navigation failed");

    // All operations should complete successfully
}

#[tokio::test]
async fn test_browser_shell_multiple_tabs_in_same_window() {
    //! Given: An initialized BrowserShell with one window
    //! When: Creating multiple tabs in the same window
    //! Then: All tabs are created and associated with the same window

    // Arrange
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell
        .initialize(config)
        .await
        .expect("Initialization failed");

    let window_id = shell
        .new_window(None)
        .await
        .expect("Window creation failed");

    // Act - create multiple tabs
    let tab1 = shell
        .new_tab(Some("https://example.com".to_string()))
        .await
        .expect("Tab 1 failed");

    let tab2 = shell
        .new_tab(Some("https://rust-lang.org".to_string()))
        .await
        .expect("Tab 2 failed");

    let tab3 = shell.new_tab(None).await.expect("Tab 3 failed");

    // Assert - all tabs created successfully
    assert_ne!(tab1, tab2);
    assert_ne!(tab2, tab3);
    assert_ne!(tab1, tab3);

    // Window should still be active
    assert_eq!(shell.active_window(), Some(window_id));
}
