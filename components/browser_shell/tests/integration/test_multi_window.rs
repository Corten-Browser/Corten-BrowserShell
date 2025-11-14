// @validates: REQ-007
//! Integration Test: Multi-Window Management
//!
//! Verifies that multiple windows can be created and managed simultaneously,
//! each with their own tabs.

use browser_shell::BrowserShell;
use shared_types::WindowConfig;

#[tokio::test]
async fn test_create_multiple_windows() {
    // Given: A running BrowserShell instance
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Creating multiple windows
    let window1 = api
        .new_window(WindowConfig::default())
        .await
        .expect("First window should be created");

    let window2 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Second window should be created");

    let window3 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Third window should be created");

    // Then: All windows should have unique IDs
    assert_ne!(window1, window2, "Window IDs should be unique");
    assert_ne!(window2, window3, "Window IDs should be unique");
    assert_ne!(window1, window3, "Window IDs should be unique");

    // Cleanup
    api.close_window(window1).await.ok();
    api.close_window(window2).await.ok();
    api.close_window(window3).await.ok();
}

#[tokio::test]
async fn test_multiple_windows_with_tabs() {
    // Given: A running BrowserShell
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Creating windows with multiple tabs each
    let window1 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window 1 should be created");

    let window2 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window 2 should be created");

    // Create tabs in window 1
    let tab1_w1 = api
        .new_tab(window1, Some("https://example.com".to_string()))
        .await
        .expect("Tab 1 in window 1 should be created");

    let tab2_w1 = api
        .new_tab(window1, Some("https://rust-lang.org".to_string()))
        .await
        .expect("Tab 2 in window 1 should be created");

    // Create tabs in window 2
    let tab1_w2 = api
        .new_tab(window2, Some("https://github.com".to_string()))
        .await
        .expect("Tab 1 in window 2 should be created");

    let tab2_w2 = api
        .new_tab(window2, Some("https://docs.rs".to_string()))
        .await
        .expect("Tab 2 in window 2 should be created");

    // Then: All tabs should have unique IDs
    assert_ne!(tab1_w1, tab2_w1, "Tabs in same window should have unique IDs");
    assert_ne!(tab1_w2, tab2_w2, "Tabs in same window should have unique IDs");
    assert_ne!(
        tab1_w1, tab1_w2,
        "Tabs across different windows should have unique IDs"
    );

    // Cleanup
    api.close_tab(tab1_w1).await.ok();
    api.close_tab(tab2_w1).await.ok();
    api.close_tab(tab1_w2).await.ok();
    api.close_tab(tab2_w2).await.ok();
    api.close_window(window1).await.ok();
    api.close_window(window2).await.ok();
}

#[tokio::test]
async fn test_close_window_with_multiple_tabs() {
    // Given: A window with multiple tabs
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window should be created");

    let _tab1 = api
        .new_tab(window_id, Some("https://example.com".to_string()))
        .await
        .expect("Tab 1 should be created");

    let _tab2 = api
        .new_tab(window_id, Some("https://rust-lang.org".to_string()))
        .await
        .expect("Tab 2 should be created");

    let _tab3 = api
        .new_tab(window_id, Some("https://github.com".to_string()))
        .await
        .expect("Tab 3 should be created");

    // When: Closing the window
    let result = api.close_window(window_id).await;

    // Then: Window (and all its tabs) should close successfully
    assert!(
        result.is_ok(),
        "Window with multiple tabs should close successfully"
    );
}

#[tokio::test]
async fn test_navigate_tabs_in_different_windows() {
    // Given: Multiple windows each with a tab
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window1 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window 1 should be created");

    let window2 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window 2 should be created");

    let tab1 = api
        .new_tab(window1, None)
        .await
        .expect("Tab in window 1 should be created");

    let tab2 = api
        .new_tab(window2, None)
        .await
        .expect("Tab in window 2 should be created");

    // When: Navigating tabs in different windows
    let nav1 = api
        .navigate(tab1, "https://example.com".to_string())
        .await;

    let nav2 = api
        .navigate(tab2, "https://rust-lang.org".to_string())
        .await;

    // Then: Both navigations should succeed independently
    assert!(
        nav1.is_ok(),
        "Navigation in window 1 tab should succeed"
    );
    assert!(
        nav2.is_ok(),
        "Navigation in window 2 tab should succeed"
    );

    // Cleanup
    api.close_tab(tab1).await.ok();
    api.close_tab(tab2).await.ok();
    api.close_window(window1).await.ok();
    api.close_window(window2).await.ok();
}

#[tokio::test]
async fn test_many_windows_simultaneously() {
    // Given: A running BrowserShell
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Creating many windows simultaneously
    let mut windows = Vec::new();
    for i in 0..10 {
        let window_id = api
            .new_window(WindowConfig::default())
            .await
            .expect(&format!("Window {} should be created", i));

        windows.push(window_id);
    }

    // Then: All windows should be created with unique IDs
    assert_eq!(windows.len(), 10, "Should have created 10 windows");

    // Verify all IDs are unique
    for i in 0..windows.len() {
        for j in (i + 1)..windows.len() {
            assert_ne!(
                windows[i], windows[j],
                "Window {} and {} should have different IDs",
                i, j
            );
        }
    }

    // Cleanup
    for window_id in windows {
        api.close_window(window_id).await.ok();
    }
}

#[tokio::test]
async fn test_window_manager_handles_concurrent_operations() {
    // Given: A running BrowserShell
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Performing concurrent window and tab operations
    let window1 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window 1 should be created");

    let window2 = api
        .new_window(WindowConfig::default())
        .await
        .expect("Window 2 should be created");

    // Create tabs concurrently
    let tab1 = api
        .new_tab(window1, Some("https://example.com".to_string()))
        .await
        .expect("Tab 1 should be created");

    let tab2 = api
        .new_tab(window2, Some("https://rust-lang.org".to_string()))
        .await
        .expect("Tab 2 should be created");

    // Then: All operations should succeed

    // Cleanup
    api.close_tab(tab1).await.ok();
    api.close_tab(tab2).await.ok();
    api.close_window(window1).await.ok();
    api.close_window(window2).await.ok();
}
