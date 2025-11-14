//! E2E test for window creation → tab creation → navigation workflow
//!
//! This test verifies the complete user workflow:
//! 1. Open browser
//! 2. Create window
//! 3. Create tab
//! 4. Navigate to URL
//! 5. Create additional tabs
//! 6. Navigate in multiple tabs
//!
//! CRITICAL: Uses REAL components throughout (no mocking)

use browser_shell::{BrowserShell, ShellConfig};
use integration_helpers::create_test_config;

#[tokio::test]
async fn test_window_tab_navigation_complete_workflow() {
    //! Complete end-to-end workflow from browser start to multi-tab navigation
    //!
    //! Flow: Init → Window → Tab → Navigate → More Tabs → More Navigation

    // Step 1: Initialize browser
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    // Step 2: Create initial window
    let window1 = shell
        .new_window(None)
        .await
        .expect("Window creation failed");
    assert_eq!(shell.active_window(), Some(window1));

    // Step 3: Create first tab
    let tab1 = shell.new_tab(None).await.expect("Tab 1 creation failed");
    assert_eq!(shell.active_tab(), Some(tab1));

    // Step 4: Navigate first tab
    shell
        .navigate("https://www.rust-lang.org".to_string())
        .await
        .expect("Navigation to Rust site failed");

    // Step 5: Create second tab
    let tab2 = shell
        .new_tab(Some("https://doc.rust-lang.org".to_string()))
        .await
        .expect("Tab 2 creation failed");
    assert_eq!(shell.active_tab(), Some(tab2));

    // Step 6: Create third tab and navigate
    let tab3 = shell.new_tab(None).await.expect("Tab 3 creation failed");
    shell
        .navigate("https://crates.io".to_string())
        .await
        .expect("Navigation to crates.io failed");

    // Step 7: Verify state
    assert_eq!(shell.active_window(), Some(window1));
    assert_eq!(shell.active_tab(), Some(tab3));

    // Step 8: Navigate active tab again
    shell
        .navigate("https://github.com/rust-lang/rust".to_string())
        .await
        .expect("Navigation to GitHub failed");

    // Cleanup
    shell.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_multiple_windows_with_tabs() {
    //! Test creating multiple windows, each with multiple tabs
    //!
    //! Simulates power user workflow with many windows and tabs

    // Initialize
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    // Window 1 with 2 tabs
    let window1 = shell
        .new_window(None)
        .await
        .expect("Window 1 creation failed");
    let w1_tab1 = shell
        .new_tab(Some("https://example.com".to_string()))
        .await
        .expect("W1T1 failed");
    let w1_tab2 = shell
        .new_tab(Some("https://example.org".to_string()))
        .await
        .expect("W1T2 failed");

    // Window 2 with 3 tabs
    let window2 = shell
        .new_window(None)
        .await
        .expect("Window 2 creation failed");
    let w2_tab1 = shell
        .new_tab(Some("https://rust-lang.org".to_string()))
        .await
        .expect("W2T1 failed");
    let w2_tab2 = shell.new_tab(None).await.expect("W2T2 failed");
    let w2_tab3 = shell
        .new_tab(Some("https://github.com".to_string()))
        .await
        .expect("W2T3 failed");

    // Verify final state
    assert_eq!(shell.active_window(), Some(window2));
    assert_eq!(shell.active_tab(), Some(w2_tab3));

    // All entities should have unique IDs
    assert_ne!(window1, window2);
    assert_ne!(w1_tab1, w1_tab2);
    assert_ne!(w2_tab1, w2_tab2);
    assert_ne!(w2_tab2, w2_tab3);

    shell.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_navigation_sequence_in_single_tab() {
    //! Test sequential navigation in a single tab
    //!
    //! Simulates user browsing through multiple pages

    // Initialize
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    // Create window and tab
    shell.new_window(None).await.expect("Window creation failed");
    let tab_id = shell.new_tab(None).await.expect("Tab creation failed");

    // Navigate through multiple URLs
    let urls = vec![
        "https://www.rust-lang.org",
        "https://doc.rust-lang.org/book/",
        "https://doc.rust-lang.org/std/",
        "https://crates.io",
        "https://github.com/rust-lang/rust",
    ];

    for url in urls {
        shell
            .navigate(url.to_string())
            .await
            .unwrap_or_else(|_| panic!("Navigation to {} failed", url));
    }

    // Tab should still be active
    assert_eq!(shell.active_tab(), Some(tab_id));

    shell.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_empty_tab_creation_then_navigation() {
    //! Test creating empty tab (no URL) then navigating
    //!
    //! Simulates user opening new tab then typing URL

    // Initialize
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    shell.new_window(None).await.expect("Window creation failed");

    // Create empty tab
    let tab_id = shell.new_tab(None).await.expect("Empty tab creation failed");

    // Now navigate (user types URL)
    shell
        .navigate("https://www.rust-lang.org".to_string())
        .await
        .expect("Navigation in empty tab failed");

    // Navigate again
    shell
        .navigate("https://doc.rust-lang.org".to_string())
        .await
        .expect("Second navigation failed");

    assert_eq!(shell.active_tab(), Some(tab_id));

    shell.shutdown().await.expect("Shutdown failed");
}

#[tokio::test]
async fn test_rapid_tab_creation_and_navigation() {
    //! Test rapid creation of many tabs with navigation
    //!
    //! Stress test for component interaction performance

    // Initialize
    let (config, _temp_dir) = create_test_config();
    let mut shell = BrowserShell::new();
    shell.initialize(config).await.expect("Initialization failed");

    shell.new_window(None).await.expect("Window creation failed");

    // Rapidly create 10 tabs and navigate
    for i in 1..=10 {
        let tab_id = shell
            .new_tab(None)
            .await
            .unwrap_or_else(|_| panic!("Tab {} creation failed", i));

        shell
            .navigate(format!("https://example.com/page{}", i))
            .await
            .unwrap_or_else(|_| panic!("Navigation {} failed", i));

        assert_eq!(shell.active_tab(), Some(tab_id));
    }

    shell.shutdown().await.expect("Shutdown failed");
}
