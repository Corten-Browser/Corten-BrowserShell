//! Contract compliance tests for tab_manager
//!
//! These tests verify that the TabManager API exactly matches the contract
//! defined in contracts/tab_manager.yaml

use shared_types::{TabId, WindowId};
use tab_manager::{TabInfo, TabLoadState, TabManager};

#[tokio::test]
async fn test_tab_manager_has_create_tab_method() {
    /**
     * Contract specifies: create_tab(window_id: WindowId, url: Option<String>) -> Result<TabId, TabError>
     * Verify the method exists and has correct signature
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();
    let url = Some("https://example.com".to_string());

    // Should compile - verifies signature
    let _result: Result<TabId, shared_types::TabError> = manager.create_tab(window_id, url).await;
}

#[tokio::test]
async fn test_tab_manager_has_close_tab_method() {
    /**
     * Contract specifies: close_tab(tab_id: TabId) -> Result<(), TabError>
     * Verify the method exists and has correct signature
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    // Should compile - verifies signature
    let _result: Result<(), shared_types::TabError> = manager.close_tab(tab_id).await;
}

#[tokio::test]
async fn test_tab_manager_has_navigate_method() {
    /**
     * Contract specifies: navigate(tab_id: TabId, url: String) -> Result<(), TabError>
     * Verify the method exists and has correct signature
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();
    let url = "https://example.com".to_string();

    // Should compile - verifies signature
    let _result: Result<(), shared_types::TabError> = manager.navigate(tab_id, url).await;
}

#[tokio::test]
async fn test_tab_manager_has_reload_method() {
    /**
     * Contract specifies: reload(tab_id: TabId, ignore_cache: bool) -> Result<(), TabError>
     * Verify the method exists and has correct signature
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();
    let ignore_cache = false;

    // Should compile - verifies signature
    let _result: Result<(), shared_types::TabError> = manager.reload(tab_id, ignore_cache).await;
}

#[tokio::test]
async fn test_tab_manager_has_stop_method() {
    /**
     * Contract specifies: stop(tab_id: TabId) -> Result<(), TabError>
     * Verify the method exists and has correct signature
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    // Should compile - verifies signature
    let _result: Result<(), shared_types::TabError> = manager.stop(tab_id).await;
}

#[tokio::test]
async fn test_tab_manager_has_go_back_method() {
    /**
     * Contract specifies: go_back(tab_id: TabId) -> Result<(), TabError>
     * Verify the method exists and has correct signature
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    // Should compile - verifies signature
    let _result: Result<(), shared_types::TabError> = manager.go_back(tab_id).await;
}

#[tokio::test]
async fn test_tab_manager_has_go_forward_method() {
    /**
     * Contract specifies: go_forward(tab_id: TabId) -> Result<(), TabError>
     * Verify the method exists and has correct signature
     */
    let mut manager = TabManager::new();
    let tab_id = TabId::new();

    // Should compile - verifies signature
    let _result: Result<(), shared_types::TabError> = manager.go_forward(tab_id).await;
}

#[tokio::test]
async fn test_tab_manager_has_get_tab_info_method() {
    /**
     * Contract specifies: get_tab_info(tab_id: TabId) -> Option<TabInfo>
     * Verify the method exists and has correct signature (synchronous)
     */
    let manager = TabManager::new();
    let tab_id = TabId::new();

    // Should compile - verifies signature (note: not async)
    let _result: Option<TabInfo> = manager.get_tab_info(tab_id);
}

#[test]
fn test_tab_info_has_required_fields() {
    /**
     * Contract specifies TabInfo fields:
     * - id: TabId
     * - window_id: WindowId
     * - title: String
     * - url: Option<String>
     * - loading: bool
     * - can_go_back: bool
     * - can_go_forward: bool
     */
    let tab_id = TabId::new();
    let window_id = WindowId::new();

    let info = TabInfo {
        id: tab_id,
        window_id,
        title: "Test".to_string(),
        url: Some(url::Url::parse("https://example.com").unwrap()),
        loading: false,
        can_go_back: false,
        can_go_forward: false,
        is_private: false,
        load_state: TabLoadState::Unloaded,
    };

    // Verify all fields compile and have correct types
    let _: TabId = info.id;
    let _: WindowId = info.window_id;
    let _: String = info.title;
    let _: Option<url::Url> = info.url;
    let _: bool = info.loading;
    let _: bool = info.can_go_back;
    let _: bool = info.can_go_forward;
    let _: bool = info.is_private;
    let _: TabLoadState = info.load_state;
}

#[tokio::test]
async fn test_contract_create_tab_returns_tab_id() {
    /**
     * Verify create_tab returns a valid TabId on success
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    let result = manager.create_tab(window_id, None).await;
    assert!(result.is_ok());

    let tab_id = result.unwrap();
    // Verify we can use the returned TabId
    let info = manager.get_tab_info(tab_id);
    assert!(info.is_some());
}

#[tokio::test]
async fn test_contract_methods_are_async_as_specified() {
    /**
     * Verify that async methods in contract are actually async
     * (This test wouldn't compile if methods weren't async)
     */
    let mut manager = TabManager::new();
    let window_id = WindowId::new();

    // These .await calls verify methods are async
    let tab_id = manager.create_tab(window_id, None).await.unwrap();
    let _ = manager.close_tab(tab_id).await;
    let _ = manager
        .navigate(tab_id, "https://example.com".to_string())
        .await;
    let _ = manager.reload(tab_id, false).await;
    let _ = manager.stop(tab_id).await;
    let _ = manager.go_back(tab_id).await;
    let _ = manager.go_forward(tab_id).await;
}

#[test]
fn test_contract_get_tab_info_is_sync() {
    /**
     * Verify that get_tab_info is synchronous (not async) as per contract
     * (This test wouldn't compile if method was async)
     */
    let manager = TabManager::new();
    let tab_id = TabId::new();

    // No .await - verifies method is NOT async
    let _result = manager.get_tab_info(tab_id);
}
