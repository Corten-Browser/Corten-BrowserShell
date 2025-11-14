// @validates: REQ-001
//! Unit tests for BrowserShell initialization

use browser_shell::BrowserShell;

#[tokio::test]
async fn test_browser_shell_new_creates_instance() {
    // Given: No prerequisites

    // When: Creating a new BrowserShell
    let browser = BrowserShell::new().await;

    // Then: Should successfully create a BrowserShell instance
    assert!(browser.is_ok(), "BrowserShell::new() should succeed");
}

#[tokio::test]
async fn test_browser_shell_initializes_all_components() {
    // Given: A new BrowserShell instance
    let browser = BrowserShell::new().await.expect("Failed to create BrowserShell");

    // When: Checking component initialization
    let status = browser.health_check().await;

    // Then: All components should be initialized and healthy
    assert!(status.is_ok(), "All components should be healthy");
}

#[tokio::test]
async fn test_browser_shell_can_shutdown_cleanly() {
    // Given: A running BrowserShell instance
    let browser = BrowserShell::new().await.expect("Failed to create BrowserShell");

    // When: Shutting down
    let result = browser.shutdown().await;

    // Then: Shutdown should complete successfully
    assert!(result.is_ok(), "Shutdown should succeed");
}
