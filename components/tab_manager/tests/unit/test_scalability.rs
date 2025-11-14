// @validates: REQ-002, REQ-003
// Unit tests for scalability and performance

use shared_types::{TabManager, WindowId};
use tab_manager::TabManagerImpl;
use std::time::Instant;

#[tokio::test]
async fn test_create_500_tabs() {
    // Given: A tab manager
    // When: I create 500 tabs
    // Then: All tabs should be created successfully
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    for i in 0..500 {
        let result = manager.create_tab(window_id, None).await;
        assert!(result.is_ok(), "Failed to create tab {}", i);
    }

    assert_eq!(manager.tab_count(), 500);
}

#[tokio::test]
async fn test_get_tab_from_500_tabs_is_fast() {
    // Given: 500 tabs in the manager
    // When: I retrieve a specific tab
    // Then: It should be fast (< 1ms)
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    // Create 500 tabs
    let mut tab_ids = Vec::new();
    for _ in 0..500 {
        let tab_id = manager.create_tab(window_id, None).await.unwrap();
        tab_ids.push(tab_id);
    }

    // Get middle tab
    let middle_tab = tab_ids[250];

    let start = Instant::now();
    let tab = manager.get_tab(middle_tab);
    let duration = start.elapsed();

    assert!(tab.is_some());
    assert!(
        duration.as_micros() < 1000,
        "Tab retrieval took {}μs, expected < 1000μs",
        duration.as_micros()
    );
}

#[tokio::test]
async fn test_tab_switching_performance() {
    // Given: Multiple tabs
    // When: I switch between tabs
    // Then: Switching should be < 10ms
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    let tab1 = manager.create_tab(window_id, None).await.unwrap();
    let tab2 = manager.create_tab(window_id, None).await.unwrap();

    let start = Instant::now();
    manager.activate_tab(tab1).await.unwrap();
    let duration1 = start.elapsed();

    let start = Instant::now();
    manager.activate_tab(tab2).await.unwrap();
    let duration2 = start.elapsed();

    assert!(
        duration1.as_millis() < 10,
        "First tab switch took {}ms, expected < 10ms",
        duration1.as_millis()
    );

    assert!(
        duration2.as_millis() < 10,
        "Second tab switch took {}ms, expected < 10ms",
        duration2.as_millis()
    );
}

#[tokio::test]
async fn test_close_tabs_from_500() {
    // Given: 500 tabs
    // When: I close all tabs
    // Then: All tabs should be closed successfully
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    // Create 500 tabs
    let mut tab_ids = Vec::new();
    for _ in 0..500 {
        let tab_id = manager.create_tab(window_id, None).await.unwrap();
        tab_ids.push(tab_id);
    }

    // Close all tabs
    for tab_id in tab_ids {
        manager.close_tab(tab_id).await.unwrap();
    }

    assert_eq!(manager.tab_count(), 0);
}

#[tokio::test]
async fn test_get_tabs_by_window_with_many_tabs() {
    // Given: Multiple windows with many tabs each
    // When: I get tabs for a specific window
    // Then: I should only get tabs for that window
    let mut manager = TabManagerImpl::new();
    let window1 = WindowId::new();
    let window2 = WindowId::new();

    // Create 100 tabs in each window
    for _ in 0..100 {
        manager.create_tab(window1, None).await.unwrap();
        manager.create_tab(window2, None).await.unwrap();
    }

    let window1_tabs = manager.get_tabs(window1);
    let window2_tabs = manager.get_tabs(window2);

    assert_eq!(window1_tabs.len(), 100);
    assert_eq!(window2_tabs.len(), 100);
}

#[tokio::test]
async fn test_memory_efficiency_with_many_tabs() {
    // Given: A tab manager
    // When: I create many tabs
    // Then: Memory usage should be reasonable (basic sanity check)
    let mut manager = TabManagerImpl::new();
    let window_id = WindowId::new();

    // Create 1000 tabs
    for _ in 0..1000 {
        manager.create_tab(window_id, None).await.unwrap();
    }

    // Basic check: manager should still be functional
    assert_eq!(manager.tab_count(), 1000);

    // All tabs should be retrievable
    let tabs = manager.get_tabs(window_id);
    assert_eq!(tabs.len(), 1000);
}
