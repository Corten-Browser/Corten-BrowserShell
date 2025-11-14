// @validates: REQ-008
//! Integration Test: Navigation History
//!
//! Verifies that back/forward navigation works correctly,
//! integrating browser_shell API with tab_manager's navigation history.

use browser_shell::BrowserShell;
use shared_types::WindowConfig;

#[tokio::test]
async fn test_navigation_history_back_and_forward() {
    // Given: A tab with navigation history
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

    // Navigate to build history
    api.navigate(tab_id, "https://rust-lang.org".to_string())
        .await
        .expect("Navigation to rust-lang should succeed");

    api.navigate(tab_id, "https://github.com".to_string())
        .await
        .expect("Navigation to github should succeed");

    // When: Going back in history
    let back_result = api.go_back(tab_id).await;

    // Then: Back navigation should succeed
    assert!(
        back_result.is_ok(),
        "Go back should succeed: {:?}",
        back_result.err()
    );

    // When: Going forward in history
    let forward_result = api.go_forward(tab_id).await;

    // Then: Forward navigation should succeed
    assert!(
        forward_result.is_ok(),
        "Go forward should succeed: {:?}",
        forward_result.err()
    );

    // Cleanup
    api.close_tab(tab_id).await.ok();
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_multiple_back_navigations() {
    // Given: A tab with multiple pages in history
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, Some("https://page1.com".to_string()))
        .await
        .expect("Tab should be created");

    // Build navigation history
    api.navigate(tab_id, "https://page2.com".to_string())
        .await
        .expect("Navigation should succeed");

    api.navigate(tab_id, "https://page3.com".to_string())
        .await
        .expect("Navigation should succeed");

    api.navigate(tab_id, "https://page4.com".to_string())
        .await
        .expect("Navigation should succeed");

    // When: Going back multiple times
    let back1 = api.go_back(tab_id).await;
    let back2 = api.go_back(tab_id).await;
    let back3 = api.go_back(tab_id).await;

    // Then: All back navigations should succeed
    assert!(back1.is_ok(), "First go back should succeed");
    assert!(back2.is_ok(), "Second go back should succeed");
    assert!(back3.is_ok(), "Third go back should succeed");

    // Cleanup
    api.close_tab(tab_id).await.ok();
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_multiple_forward_navigations() {
    // Given: A tab with navigation history and back navigations
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, Some("https://page1.com".to_string()))
        .await
        .expect("Tab should be created");

    // Build navigation history
    api.navigate(tab_id, "https://page2.com".to_string())
        .await
        .expect("Navigation should succeed");

    api.navigate(tab_id, "https://page3.com".to_string())
        .await
        .expect("Navigation should succeed");

    // Go back to create forward history
    api.go_back(tab_id).await.expect("Go back should succeed");
    api.go_back(tab_id).await.expect("Go back should succeed");

    // When: Going forward multiple times
    let forward1 = api.go_forward(tab_id).await;
    let forward2 = api.go_forward(tab_id).await;

    // Then: All forward navigations should succeed
    assert!(forward1.is_ok(), "First go forward should succeed");
    assert!(forward2.is_ok(), "Second go forward should succeed");

    // Cleanup
    api.close_tab(tab_id).await.ok();
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_back_forward_back_pattern() {
    // Given: A tab with navigation history
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, Some("https://page1.com".to_string()))
        .await
        .expect("Tab should be created");

    api.navigate(tab_id, "https://page2.com".to_string())
        .await
        .expect("Navigation should succeed");

    api.navigate(tab_id, "https://page3.com".to_string())
        .await
        .expect("Navigation should succeed");

    // When: Performing back-forward-back pattern
    let back1 = api.go_back(tab_id).await;
    let forward1 = api.go_forward(tab_id).await;
    let back2 = api.go_back(tab_id).await;

    // Then: All operations should succeed
    assert!(back1.is_ok(), "First back should succeed");
    assert!(forward1.is_ok(), "Forward should succeed");
    assert!(back2.is_ok(), "Second back should succeed");

    // Cleanup
    api.close_tab(tab_id).await.ok();
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_navigation_history_per_tab() {
    // Given: Multiple tabs with independent navigation histories
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab1 = api
        .new_tab(window_id, Some("https://tab1-page1.com".to_string()))
        .await
        .expect("Tab 1 should be created");

    let tab2 = api
        .new_tab(window_id, Some("https://tab2-page1.com".to_string()))
        .await
        .expect("Tab 2 should be created");

    // Build different histories for each tab
    api.navigate(tab1, "https://tab1-page2.com".to_string())
        .await
        .expect("Tab 1 navigation should succeed");

    api.navigate(tab2, "https://tab2-page2.com".to_string())
        .await
        .expect("Tab 2 navigation should succeed");

    api.navigate(tab2, "https://tab2-page3.com".to_string())
        .await
        .expect("Tab 2 navigation should succeed");

    // When: Going back in each tab
    let back_tab1 = api.go_back(tab1).await;
    let back_tab2 = api.go_back(tab2).await;

    // Then: Both should succeed independently
    assert!(
        back_tab1.is_ok(),
        "Tab 1 back navigation should succeed"
    );
    assert!(
        back_tab2.is_ok(),
        "Tab 2 back navigation should succeed"
    );

    // Cleanup
    api.close_tab(tab1).await.ok();
    api.close_tab(tab2).await.ok();
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_reload_maintains_history() {
    // Given: A tab with navigation history
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, Some("https://page1.com".to_string()))
        .await
        .expect("Tab should be created");

    api.navigate(tab_id, "https://page2.com".to_string())
        .await
        .expect("Navigation should succeed");

    // When: Reloading the current page
    let reload_result = api.reload(tab_id).await;

    // Then: Reload should succeed
    assert!(reload_result.is_ok(), "Reload should succeed");

    // And: Back navigation should still work (history preserved)
    let back_result = api.go_back(tab_id).await;
    assert!(
        back_result.is_ok(),
        "History should be preserved after reload"
    );

    // Cleanup
    api.close_tab(tab_id).await.ok();
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_navigation_after_going_back() {
    // Given: A tab with history, after going back
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let tab_id = api
        .new_tab(window_id, Some("https://page1.com".to_string()))
        .await
        .expect("Tab should be created");

    api.navigate(tab_id, "https://page2.com".to_string())
        .await
        .expect("Navigation should succeed");

    api.go_back(tab_id)
        .await
        .expect("Go back should succeed");

    // When: Navigating to a new page (should clear forward history)
    let nav_result = api.navigate(tab_id, "https://page3.com".to_string()).await;

    // Then: Navigation should succeed
    assert!(
        nav_result.is_ok(),
        "Navigation after going back should succeed"
    );

    // Cleanup
    api.close_tab(tab_id).await.ok();
    api.close_window(window_id).await.ok();
}
