// @validates: REQ-002, REQ-003
// Unit tests for tab manager basic operations

use shared_types::{TabManager, TabId, WindowId, Url, TabError};
use tab_manager::TabManagerImpl;

#[tokio::test]
async fn test_create_tab_manager() {
    // Given: I want to create a new tab manager
    // When: I create a TabManagerImpl
    // Then: It should be successfully created
    let manager = TabManagerImpl::new();

    // Should have no tabs initially
    assert_eq!(manager.tab_count(), 0);
}

#[tokio::test]
async fn test_create_tab_in_window() {
    // Given: A tab manager and a window
    // When: I create a tab in that window
    // Then: The tab should be created with a unique ID
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None)
        .await
        .expect("Tab creation should succeed");

    // Tab should exist
    assert!(manager.get_tab(tab_id).is_some());

    // Tab count should be 1
    assert_eq!(manager.tab_count(), 1);
}

#[tokio::test]
async fn test_create_tab_with_url() {
    // Given: A tab manager and a URL
    // When: I create a tab with that URL
    // Then: The tab should have that URL
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();
    let url = Url::parse("https://example.com").expect("Valid URL");

    let tab_id = manager.create_tab(window_id, Some(url.clone()))
        .await
        .expect("Tab creation should succeed");

    let tab = manager.get_tab(tab_id).expect("Tab should exist");
    assert_eq!(tab.url.as_ref().map(|u| u.as_str()), Some("https://example.com"));
}

#[tokio::test]
async fn test_close_tab() {
    // Given: A tab manager with a tab
    // When: I close the tab
    // Then: The tab should no longer exist
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None)
        .await
        .expect("Tab creation should succeed");

    // Close the tab
    manager.close_tab(tab_id)
        .await
        .expect("Tab closure should succeed");

    // Tab should no longer exist
    assert!(manager.get_tab(tab_id).is_none());
    assert_eq!(manager.tab_count(), 0);
}

#[tokio::test]
async fn test_close_nonexistent_tab_fails() {
    // Given: A tab manager
    // When: I try to close a tab that doesn't exist
    // Then: I should get an error
    let mut manager = TabManagerImpl::new();
    let fake_tab_id = TabId::new();

    let result = manager.close_tab(fake_tab_id).await;
    assert!(result.is_err());

    match result {
        Err(TabError::NotFound(id)) => assert_eq!(id, fake_tab_id),
        _ => panic!("Expected TabError::NotFound"),
    }
}

#[tokio::test]
async fn test_get_tabs_by_window() {
    // Given: Multiple tabs in different windows
    // When: I get tabs for a specific window
    // Then: I should only get tabs for that window
    let mut manager = TabManagerImpl::new();
    let window1 = WindowId::new();
    let window2 = WindowId::new();

    let tab1 = manager.create_tab(window1, None).await.unwrap();
    let tab2 = manager.create_tab(window1, None).await.unwrap();
    let _tab3 = manager.create_tab(window2, None).await.unwrap();

    let window1_tabs = manager.get_tabs(window1);
    assert_eq!(window1_tabs.len(), 2);

    let tab_ids: Vec<TabId> = window1_tabs.iter().map(|t| t.id).collect();
    assert!(tab_ids.contains(&tab1));
    assert!(tab_ids.contains(&tab2));
}

#[tokio::test]
async fn test_navigate_tab() {
    // Given: A tab
    // When: I navigate to a URL
    // Then: The tab's URL should be updated
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await.unwrap();

    let url = Url::parse("https://example.com").unwrap();
    manager.navigate(tab_id, url.clone()).await.unwrap();

    let tab = manager.get_tab(tab_id).unwrap();
    assert_eq!(tab.url.as_ref().map(|u| u.as_str()), Some("https://example.com"));
}

#[tokio::test]
async fn test_navigate_nonexistent_tab_fails() {
    // Given: A tab manager
    // When: I try to navigate a nonexistent tab
    // Then: I should get an error
    let mut manager = TabManagerImpl::new();
    let fake_tab_id = TabId::new();
    let url = Url::parse("https://example.com").unwrap();

    let result = manager.navigate(fake_tab_id, url).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_reload_tab() {
    // Given: A tab with a URL
    // When: I reload the tab
    // Then: The reload should succeed (implementation detail: just marks loading)
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();
    let url = Url::parse("https://example.com").unwrap();

    let tab_id = manager.create_tab(window_id, Some(url)).await.unwrap();

    // Reload should succeed
    let result = manager.reload(tab_id, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stop_loading() {
    // Given: A tab
    // When: I stop loading
    // Then: The operation should succeed
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await.unwrap();

    let result = manager.stop(tab_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_activate_tab() {
    // Given: Multiple tabs in a window
    // When: I activate a specific tab
    // Then: That tab should be marked as active
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let _tab1 = manager.create_tab(window_id, None).await.unwrap();
    let tab2 = manager.create_tab(window_id, None).await.unwrap();

    // Activate tab2
    manager.activate_tab(tab2).await.unwrap();

    // tab2 should be the active tab (implementation-specific check)
    assert!(manager.is_active_tab(tab2));
}
