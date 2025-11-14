//! Unit tests for TabManager

use shared_types::{TabError, TabId, WindowId};
use tab_manager::TabManager;
use url::Url;

#[tokio::test]
async fn test_create_tab_manager() {
    /**
     * Given no preconditions
     * When creating a new TabManager
     * Then it should be created successfully
     */
    let manager = TabManager::new();
    assert_eq!(manager.tab_count(), 0);
}

#[tokio::test]
async fn test_create_tab_with_url() {
    /**
     * Given a window ID and URL
     * When creating a new tab
     * Then the tab should be created and returned
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();
    let url = "https://example.com".to_string();

    let tab_id = manager.create_tab(window_id, Some(url.clone())).await;

    assert!(tab_id.is_ok());
    let tab_id = tab_id.unwrap();

    let tab_info = manager.get_tab_info(tab_id);
    assert!(tab_info.is_some());

    let tab_info = tab_info.unwrap();
    assert_eq!(tab_info.id, tab_id);
    assert_eq!(tab_info.window_id, window_id);
    // URL parser normalizes URLs (adds trailing slash)
    assert_eq!(
        tab_info.url.as_ref().unwrap().as_str(),
        "https://example.com/"
    );
}

#[tokio::test]
async fn test_create_tab_without_url() {
    /**
     * Given a window ID and no URL
     * When creating a new tab
     * Then an empty tab should be created
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await;

    assert!(tab_id.is_ok());
    let tab_id = tab_id.unwrap();

    let tab_info = manager.get_tab_info(tab_id);
    assert!(tab_info.is_some());

    let tab_info = tab_info.unwrap();
    assert_eq!(tab_info.id, tab_id);
    assert_eq!(tab_info.window_id, window_id);
    assert!(tab_info.url.is_none());
}

#[tokio::test]
async fn test_close_tab() {
    /**
     * Given an existing tab
     * When closing the tab
     * Then the tab should be removed from the manager
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await.unwrap();
    assert!(manager.get_tab_info(tab_id).is_some());

    let result = manager.close_tab(tab_id).await;
    assert!(result.is_ok());

    assert!(manager.get_tab_info(tab_id).is_none());
}

#[tokio::test]
async fn test_close_nonexistent_tab() {
    /**
     * Given a non-existent tab ID
     * When attempting to close the tab
     * Then an error should be returned
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    let result = manager.close_tab(tab_id).await;
    assert!(result.is_err());

    match result {
        Err(TabError::NotFound(id)) => assert_eq!(id, tab_id),
        _ => panic!("Expected TabError::NotFound"),
    }
}

#[tokio::test]
async fn test_navigate_to_url() {
    /**
     * Given an existing tab
     * When navigating to a new URL
     * Then the tab's URL should be updated
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    let result = manager
        .navigate(tab_id, "https://example.org".to_string())
        .await;
    assert!(result.is_ok());

    let tab_info = manager.get_tab_info(tab_id).unwrap();
    assert_eq!(
        tab_info.url.as_ref().unwrap().as_str(),
        "https://example.org/"
    );
    assert_eq!(tab_info.can_go_back, true);
    assert_eq!(tab_info.can_go_forward, false);
}

#[tokio::test]
async fn test_navigate_invalid_url() {
    /**
     * Given an existing tab
     * When navigating to an invalid URL
     * Then an error should be returned
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await.unwrap();

    let result = manager
        .navigate(tab_id, "not a valid url".to_string())
        .await;
    assert!(result.is_err());

    match result {
        Err(TabError::NavigationFailed(_)) => (),
        _ => panic!("Expected TabError::NavigationFailed"),
    }
}

#[tokio::test]
async fn test_navigate_nonexistent_tab() {
    /**
     * Given a non-existent tab ID
     * When attempting to navigate
     * Then an error should be returned
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    let result = manager
        .navigate(tab_id, "https://example.com".to_string())
        .await;
    assert!(result.is_err());

    match result {
        Err(TabError::NotFound(id)) => assert_eq!(id, tab_id),
        _ => panic!("Expected TabError::NotFound"),
    }
}

#[tokio::test]
async fn test_reload_tab() {
    /**
     * Given an existing tab with a URL
     * When reloading the tab
     * Then the reload should succeed
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    let result = manager.reload(tab_id, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reload_with_cache_ignore() {
    /**
     * Given an existing tab
     * When reloading with cache ignore flag
     * Then the reload should succeed
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    let result = manager.reload(tab_id, true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reload_nonexistent_tab() {
    /**
     * Given a non-existent tab ID
     * When attempting to reload
     * Then an error should be returned
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    let result = manager.reload(tab_id, false).await;
    assert!(result.is_err());

    match result {
        Err(TabError::NotFound(id)) => assert_eq!(id, tab_id),
        _ => panic!("Expected TabError::NotFound"),
    }
}

#[tokio::test]
async fn test_stop_loading() {
    /**
     * Given a loading tab
     * When stopping the load
     * Then the loading should stop
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    let result = manager.stop(tab_id).await;
    assert!(result.is_ok());

    let tab_info = manager.get_tab_info(tab_id).unwrap();
    assert_eq!(tab_info.loading, false);
}

#[tokio::test]
async fn test_stop_nonexistent_tab() {
    /**
     * Given a non-existent tab ID
     * When attempting to stop loading
     * Then an error should be returned
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    let result = manager.stop(tab_id).await;
    assert!(result.is_err());

    match result {
        Err(TabError::NotFound(id)) => assert_eq!(id, tab_id),
        _ => panic!("Expected TabError::NotFound"),
    }
}

#[tokio::test]
async fn test_go_back() {
    /**
     * Given a tab with navigation history
     * When going back
     * Then the previous URL should be loaded
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    manager
        .navigate(tab_id, "https://example.org".to_string())
        .await
        .unwrap();

    let result = manager.go_back(tab_id).await;
    assert!(result.is_ok());

    let tab_info = manager.get_tab_info(tab_id).unwrap();
    assert_eq!(
        tab_info.url.as_ref().unwrap().as_str(),
        "https://example.com/"
    );
    assert_eq!(tab_info.can_go_back, false);
    assert_eq!(tab_info.can_go_forward, true);
}

#[tokio::test]
async fn test_go_back_no_history() {
    /**
     * Given a tab with no back history
     * When attempting to go back
     * Then an error should be returned
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    let result = manager.go_back(tab_id).await;
    assert!(result.is_err());

    match result {
        Err(TabError::NavigationFailed(_)) => (),
        _ => panic!("Expected TabError::NavigationFailed"),
    }
}

#[tokio::test]
async fn test_go_forward() {
    /**
     * Given a tab that has gone back in history
     * When going forward
     * Then the next URL should be loaded
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    manager
        .navigate(tab_id, "https://example.org".to_string())
        .await
        .unwrap();
    manager.go_back(tab_id).await.unwrap();

    let result = manager.go_forward(tab_id).await;
    assert!(result.is_ok());

    let tab_info = manager.get_tab_info(tab_id).unwrap();
    assert_eq!(
        tab_info.url.as_ref().unwrap().as_str(),
        "https://example.org/"
    );
    assert_eq!(tab_info.can_go_back, true);
    assert_eq!(tab_info.can_go_forward, false);
}

#[tokio::test]
async fn test_go_forward_no_history() {
    /**
     * Given a tab with no forward history
     * When attempting to go forward
     * Then an error should be returned
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    let result = manager.go_forward(tab_id).await;
    assert!(result.is_err());

    match result {
        Err(TabError::NavigationFailed(_)) => (),
        _ => panic!("Expected TabError::NavigationFailed"),
    }
}

#[tokio::test]
async fn test_get_tab_info_nonexistent() {
    /**
     * Given a non-existent tab ID
     * When getting tab info
     * Then None should be returned
     */
    let manager = TabManager::new();
    let tab_id = TabId::new();

    let result = manager.get_tab_info(tab_id);
    assert!(result.is_none());
}

#[tokio::test]
async fn test_multiple_tabs_same_window() {
    /**
     * Given multiple tabs in the same window
     * When creating and managing tabs
     * Then each tab should maintain its own state
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab1 = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();
    let tab2 = manager
        .create_tab(window_id, Some("https://example.org".to_string()))
        .await
        .unwrap();

    assert_ne!(tab1, tab2);

    let info1 = manager.get_tab_info(tab1).unwrap();
    let info2 = manager.get_tab_info(tab2).unwrap();

    assert_eq!(info1.url.as_ref().unwrap().as_str(), "https://example.com/");
    assert_eq!(info2.url.as_ref().unwrap().as_str(), "https://example.org/");
}

#[tokio::test]
async fn test_navigation_history_cleared_on_new_navigate() {
    /**
     * Given a tab with back/forward history
     * When navigating to a new URL (not back/forward)
     * Then the forward history should be cleared
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let tab_id = manager
        .create_tab(window_id, Some("https://example.com".to_string()))
        .await
        .unwrap();

    manager
        .navigate(tab_id, "https://example.org".to_string())
        .await
        .unwrap();
    manager
        .navigate(tab_id, "https://example.net".to_string())
        .await
        .unwrap();

    manager.go_back(tab_id).await.unwrap();
    let info = manager.get_tab_info(tab_id).unwrap();
    assert_eq!(info.can_go_forward, true);

    // Navigate to new URL - should clear forward history
    manager
        .navigate(tab_id, "https://new-url.com".to_string())
        .await
        .unwrap();

    let info = manager.get_tab_info(tab_id).unwrap();
    assert_eq!(info.can_go_forward, false);
    assert_eq!(info.can_go_back, true);
}
