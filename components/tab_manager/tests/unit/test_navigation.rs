// @validates: REQ-003
// Unit tests for navigation history

use shared_types::{TabManager, WindowId, Url};
use tab_manager::TabManagerImpl;

#[tokio::test]
async fn test_navigation_back_forward_initially_disabled() {
    // Given: A newly created tab
    // When: I check back/forward availability
    // Then: Both should be disabled
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await.unwrap();

    let tab = manager.get_tab(tab_id).unwrap();
    assert_eq!(tab.can_go_back, false);
    assert_eq!(tab.can_go_forward, false);
}

#[tokio::test]
async fn test_navigation_back_enabled_after_navigate() {
    // Given: A tab with initial URL
    // When: I navigate to a new URL
    // Then: Back should be enabled
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();
    let url1 = Url::parse("https://example.com").unwrap();

    let tab_id = manager.create_tab(window_id, Some(url1)).await.unwrap();

    // Navigate to second URL
    let url2 = Url::parse("https://example.com/page2").unwrap();
    manager.navigate(tab_id, url2).await.unwrap();

    let tab = manager.get_tab(tab_id).unwrap();
    assert!(tab.can_go_back, "Should be able to go back after navigation");
    assert!(!tab.can_go_forward, "Should not be able to go forward");
}

#[tokio::test]
async fn test_go_back() {
    // Given: A tab with navigation history
    // When: I go back
    // Then: The tab should navigate to previous URL
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();
    let url1 = Url::parse("https://example.com").unwrap();

    let tab_id = manager.create_tab(window_id, Some(url1.clone())).await.unwrap();

    let url2 = Url::parse("https://example.com/page2").unwrap();
    manager.navigate(tab_id, url2).await.unwrap();

    // Go back
    manager.go_back(tab_id).await.unwrap();

    let tab = manager.get_tab(tab_id).unwrap();
    assert_eq!(tab.url.as_ref().map(|u| u.as_str()), Some("https://example.com"));
}

#[tokio::test]
async fn test_go_forward() {
    // Given: A tab with forward history
    // When: I go forward
    // Then: The tab should navigate to next URL
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();
    let url1 = Url::parse("https://example.com").unwrap();

    let tab_id = manager.create_tab(window_id, Some(url1)).await.unwrap();

    let url2 = Url::parse("https://example.com/page2").unwrap();
    manager.navigate(tab_id, url2.clone()).await.unwrap();

    // Go back first
    manager.go_back(tab_id).await.unwrap();

    // Now forward should be enabled
    let tab = manager.get_tab(tab_id).unwrap();
    assert!(tab.can_go_forward);

    // Go forward
    manager.go_forward(tab_id).await.unwrap();

    let tab = manager.get_tab(tab_id).unwrap();
    assert_eq!(tab.url.as_ref().map(|u| u.as_str()), Some("https://example.com/page2"));
}

#[tokio::test]
async fn test_go_back_without_history_fails() {
    // Given: A tab without back history
    // When: I try to go back
    // Then: I should get an error
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await.unwrap();

    let result = manager.go_back(tab_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_go_forward_without_history_fails() {
    // Given: A tab without forward history
    // When: I try to go forward
    // Then: I should get an error
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab_id = manager.create_tab(window_id, None).await.unwrap();

    let result = manager.go_forward(tab_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_navigation_clears_forward_history() {
    // Given: A tab with forward history
    // When: I navigate to a new URL
    // Then: Forward history should be cleared
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();
    let url1 = Url::parse("https://example.com").unwrap();

    let tab_id = manager.create_tab(window_id, Some(url1)).await.unwrap();

    let url2 = Url::parse("https://example.com/page2").unwrap();
    manager.navigate(tab_id, url2).await.unwrap();

    // Go back
    manager.go_back(tab_id).await.unwrap();

    // Now navigate to a new URL (should clear forward history)
    let url3 = Url::parse("https://example.com/page3").unwrap();
    manager.navigate(tab_id, url3).await.unwrap();

    let tab = manager.get_tab(tab_id).unwrap();
    assert!(!tab.can_go_forward, "Forward history should be cleared");
}

#[tokio::test]
async fn test_multiple_back_forward_navigations() {
    // Given: A tab with multiple URLs in history
    // When: I navigate back and forward multiple times
    // Then: URLs should be correct
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();
    let url1 = Url::parse("https://example.com/1").unwrap();

    let tab_id = manager.create_tab(window_id, Some(url1)).await.unwrap();

    // Navigate through multiple URLs
    for i in 2..=5 {
        let url = Url::parse(&format!("https://example.com/{}", i)).unwrap();
        manager.navigate(tab_id, url).await.unwrap();
    }

    // Should be at URL 5
    assert_eq!(
        manager.get_tab(tab_id).unwrap().url.as_ref().map(|u| u.as_str()),
        Some("https://example.com/5")
    );

    // Go back twice (should be at URL 3)
    manager.go_back(tab_id).await.unwrap();
    manager.go_back(tab_id).await.unwrap();

    assert_eq!(
        manager.get_tab(tab_id).unwrap().url.as_ref().map(|u| u.as_str()),
        Some("https://example.com/3")
    );

    // Go forward once (should be at URL 4)
    manager.go_forward(tab_id).await.unwrap();

    assert_eq!(
        manager.get_tab(tab_id).unwrap().url.as_ref().map(|u| u.as_str()),
        Some("https://example.com/4")
    );
}
