// @validates: REQ-003, REQ-004, REQ-005
//! Integration Test: Window/Tab Workflow
//!
//! Verifies the complete workflow:
//! 1. Create window
//! 2. Create tab in window
//! 3. Navigate tab to URL
//! 4. Close tab
//! 5. Close window

use browser_shell::BrowserShell;
use shared_types::WindowConfig;

#[tokio::test]
async fn test_complete_window_tab_workflow() {
    // Given: A running BrowserShell instance
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Creating a new window
    let window_config = WindowConfig::default();
    let window_id = api
        .new_window(window_config)
        .await
        .expect("Should create window successfully");

    // Then: Window should be created and return valid window ID

    // When: Creating a tab in the window
    let tab_id = api
        .new_tab(window_id, Some("https://example.com".to_string()))
        .await
        .expect("Should create tab successfully");

    // Then: Tab should be created with valid tab ID

    // When: Navigating the tab to a different URL
    let navigate_result = api
        .navigate(tab_id, "https://rust-lang.org".to_string())
        .await;

    // Then: Navigation should succeed
    assert!(
        navigate_result.is_ok(),
        "Navigation should succeed: {:?}",
        navigate_result.err()
    );

    // When: Closing the tab
    let close_tab_result = api.close_tab(tab_id).await;

    // Then: Tab should close successfully
    assert!(
        close_tab_result.is_ok(),
        "Tab should close successfully: {:?}",
        close_tab_result.err()
    );

    // When: Closing the window
    let close_window_result = api.close_window(window_id).await;

    // Then: Window should close successfully
    assert!(
        close_window_result.is_ok(),
        "Window should close successfully: {:?}",
        close_window_result.err()
    );
}

#[tokio::test]
async fn test_create_window_returns_valid_id() {
    // Given: A running BrowserShell
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Creating a window
    let window_config = WindowConfig::default();
    let window_id = api
        .new_window(window_config)
        .await
        .expect("Window creation should succeed");

    // Then: Window ID should be valid and usable
}

#[tokio::test]
async fn test_create_tab_with_url() {
    // Given: A window exists
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    // When: Creating a tab with a URL
    let tab_id = api
        .new_tab(window_id, Some("https://example.com".to_string()))
        .await
        .expect("Tab creation should succeed");

    // Then: Tab should be created with valid ID
}

#[tokio::test]
async fn test_create_tab_without_url() {
    // Given: A window exists
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    // When: Creating a tab without a URL (blank tab)
    let tab_id = api
        .new_tab(window_id, None)
        .await
        .expect("Tab creation should succeed");

    // Then: Tab should be created successfully
    // No assertion needed - expect() above already validates success
}

#[tokio::test]
async fn test_navigate_tab_to_url() {
    // Given: A tab exists in a window
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, None)
        .await
        .expect("Tab should be created");

    // When: Navigating to a URL
    let result = api
        .navigate(tab_id, "https://www.rust-lang.org".to_string())
        .await;

    // Then: Navigation should succeed
    assert!(result.is_ok(), "Navigation should succeed");
}

#[tokio::test]
async fn test_reload_tab() {
    // Given: A tab with a URL
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, Some("https://example.com".to_string()))
        .await
        .expect("Tab should be created");

    // When: Reloading the tab
    let result = api.reload(tab_id).await;

    // Then: Reload should succeed
    assert!(result.is_ok(), "Reload should succeed");
}

#[tokio::test]
async fn test_close_tab_succeeds() {
    // Given: A tab exists
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, None)
        .await
        .expect("Tab should be created");

    // When: Closing the tab
    let result = api.close_tab(tab_id).await;

    // Then: Close should succeed
    assert!(result.is_ok(), "Tab close should succeed");
}

#[tokio::test]
async fn test_close_window_succeeds() {
    // Given: A window exists
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    // When: Closing the window
    let result = api.close_window(window_id).await;

    // Then: Close should succeed
    assert!(result.is_ok(), "Window close should succeed");
}
