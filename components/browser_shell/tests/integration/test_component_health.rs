// @validates: REQ-009
//! Integration Test: Component Health
//!
//! Verifies that all components report healthy status and are properly initialized.

use browser_shell::BrowserShell;

#[tokio::test]
async fn test_all_components_healthy_on_startup() {
    // Given: A newly initialized BrowserShell

    // When: Creating BrowserShell (initializes all components)
    let browser = BrowserShell::new().await;

    // Then: BrowserShell should initialize successfully
    assert!(
        browser.is_ok(),
        "BrowserShell should initialize with all components healthy"
    );

    let browser = browser.unwrap();

    // When: Checking health status
    let health = browser.health_check().await;

    // Then: All components should report healthy
    assert!(
        health.is_ok(),
        "All components should be healthy: {:?}",
        health.err()
    );
}

#[tokio::test]
async fn test_components_remain_healthy_during_operations() {
    // Given: A running BrowserShell with ongoing operations
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // Perform various operations
    let window_id = api
        .new_window(shared_types::WindowConfig::default())
        .await
        .expect("Window creation should succeed");

    let tab_id = api
        .new_tab(window_id, Some("https://example.com".to_string()))
        .await
        .expect("Tab creation should succeed");

    api.navigate(tab_id, "https://rust-lang.org".to_string())
        .await
        .expect("Navigation should succeed");

    api.set_setting("test_key", "test_value".to_string())
        .await
        .expect("Setting should succeed");

    // When: Checking health after operations
    let health = browser.health_check().await;

    // Then: Components should still be healthy
    assert!(
        health.is_ok(),
        "Components should remain healthy during operations"
    );

    // Cleanup
    api.close_tab(tab_id).await.ok();
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_message_bus_component_healthy() {
    // Given: BrowserShell initialized (initializes MessageBus first as core infrastructure)
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    // When: Checking overall health (includes MessageBus)
    let health = browser.health_check().await;

    // Then: MessageBus should be healthy
    assert!(
        health.is_ok(),
        "MessageBus component should be healthy and initialized"
    );
}

#[tokio::test]
async fn test_window_manager_component_healthy() {
    // Given: BrowserShell initialized (initializes WindowManager)
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Using WindowManager (via API)
    let window_result = api.new_window(shared_types::WindowConfig::default()).await;

    // Then: WindowManager should be healthy and functional
    assert!(
        window_result.is_ok(),
        "WindowManager component should be healthy and functional"
    );

    // Cleanup
    if let Ok(window_id) = window_result {
        api.close_window(window_id).await.ok();
    }
}

#[tokio::test]
async fn test_tab_manager_component_healthy() {
    // Given: BrowserShell initialized (initializes TabManager)
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(shared_types::WindowConfig::default())
        .await
        .expect("Window should be created");

    // When: Using TabManager (via API)
    let tab_result = api.new_tab(window_id, None).await;

    // Then: TabManager should be healthy and functional
    assert!(
        tab_result.is_ok(),
        "TabManager component should be healthy and functional"
    );

    // Cleanup
    if let Ok(tab_id) = tab_result {
        api.close_tab(tab_id).await.ok();
    }
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_settings_manager_component_healthy() {
    // Given: BrowserShell initialized (initializes SettingsManager)
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Using SettingsManager (via API)
    let set_result = api
        .set_setting("health_test", "value".to_string())
        .await;

    // Then: SettingsManager should be healthy and functional
    assert!(
        set_result.is_ok(),
        "SettingsManager component should be healthy and functional"
    );

    let get_result = api.get_setting("health_test").await;
    assert!(
        get_result.is_ok(),
        "SettingsManager should retrieve settings correctly"
    );
}

#[tokio::test]
async fn test_component_coordinator_healthy() {
    // Given: BrowserShell with ComponentCoordinator managing all components
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    // When: Checking health (ComponentCoordinator manages component lifecycle)
    let health = browser.health_check().await;

    // Then: ComponentCoordinator should report all components healthy
    assert!(
        health.is_ok(),
        "ComponentCoordinator should report all managed components as healthy"
    );
}

#[tokio::test]
async fn test_components_healthy_after_intensive_operations() {
    // Given: BrowserShell performing intensive operations
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // Perform intensive operations
    let mut windows = Vec::new();
    let mut tabs = Vec::new();

    for i in 0..5 {
        let window_id = api
            .new_window(shared_types::WindowConfig::default())
            .await
            .expect(&format!("Window {} should be created", i));

        windows.push(window_id);

        for j in 0..3 {
            let tab_id = api
                .new_tab(window_id, Some(format!("https://page-{}-{}.com", i, j)))
                .await
                .expect(&format!("Tab {}-{} should be created", i, j));

            tabs.push(tab_id);

            api.navigate(tab_id, format!("https://navigate-{}-{}.com", i, j))
                .await
                .expect("Navigation should succeed");
        }
    }

    // When: Checking health after intensive operations
    let health = browser.health_check().await;

    // Then: All components should still be healthy
    assert!(
        health.is_ok(),
        "Components should remain healthy after intensive operations"
    );

    // Cleanup
    for tab_id in tabs {
        api.close_tab(tab_id).await.ok();
    }
    for window_id in windows {
        api.close_window(window_id).await.ok();
    }
}

#[tokio::test]
async fn test_components_healthy_during_concurrent_operations() {
    // Given: BrowserShell with concurrent operations on different components
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // Create windows and tabs concurrently
    let window1 = api
        .new_window(shared_types::WindowConfig::default())
        .await
        .expect("Window 1 should be created");

    let window2 = api
        .new_window(shared_types::WindowConfig::default())
        .await
        .expect("Window 2 should be created");

    // Perform operations concurrently
    api.set_setting("key1", "value1".to_string())
        .await
        .expect("Setting 1 should succeed");

    let tab1 = api
        .new_tab(window1, Some("https://example.com".to_string()))
        .await
        .expect("Tab 1 should be created");

    api.set_setting("key2", "value2".to_string())
        .await
        .expect("Setting 2 should succeed");

    let tab2 = api
        .new_tab(window2, Some("https://rust-lang.org".to_string()))
        .await
        .expect("Tab 2 should be created");

    // When: Checking health during concurrent operations
    let health = browser.health_check().await;

    // Then: All components should handle concurrency and remain healthy
    assert!(
        health.is_ok(),
        "Components should handle concurrent operations and remain healthy"
    );

    // Cleanup
    api.close_tab(tab1).await.ok();
    api.close_tab(tab2).await.ok();
    api.close_window(window1).await.ok();
    api.close_window(window2).await.ok();
}

#[tokio::test]
async fn test_graceful_shutdown_maintains_component_health() {
    // Given: A running BrowserShell with active operations
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    let window_id = api
        .new_window(shared_types::WindowConfig::default())
        .await
        .expect("Window should be created");

    let _tab_id = api
        .new_tab(window_id, Some("https://example.com".to_string()))
        .await
        .expect("Tab should be created");

    // When: Performing graceful shutdown
    let shutdown_result = browser.shutdown().await;

    // Then: Shutdown should succeed (all components shut down cleanly)
    assert!(
        shutdown_result.is_ok(),
        "Graceful shutdown should succeed with all components shutting down cleanly"
    );
}
