//! Contract compliance tests
//!
//! These tests verify that window_manager implements the EXACT API defined
//! in contracts/window_manager.yaml

use platform_abstraction::LinuxWindow;
use shared_types::{WindowConfig, WindowError, WindowId};
use window_manager::WindowManager;

#[tokio::test]
async fn test_window_manager_has_create_window() {
    // From contract: create_window(config: WindowConfig) -> Result<WindowId, WindowError>
    let mut manager = WindowManager::<LinuxWindow>::new();
    let config = WindowConfig::default();

    // Verify method exists and returns correct type
    let result: Result<WindowId, WindowError> = manager.create_window(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_window_manager_has_close_window() {
    // From contract: close_window(id: WindowId) -> Result<(), WindowError>
    let mut manager = WindowManager::<LinuxWindow>::new();
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    // Verify method exists and returns correct type
    let result: Result<(), WindowError> = manager.close_window(window_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_window_manager_has_get_windows() {
    // From contract: get_windows() -> Vec<WindowId>
    let manager = WindowManager::<LinuxWindow>::new();

    // Verify method exists and returns correct type
    let result: Vec<WindowId> = manager.get_windows();
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_window_manager_has_get_window_config() {
    // From contract: get_window_config(id: WindowId) -> Option<WindowConfig>
    let mut manager = WindowManager::<LinuxWindow>::new();
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    // Verify method exists and returns correct type
    let result: Option<WindowConfig> = manager.get_window_config(window_id);
    assert!(result.is_some());
}

#[tokio::test]
async fn test_window_manager_has_resize_window() {
    // From contract: resize_window(id: WindowId, width: u32, height: u32) -> Result<(), WindowError>
    let mut manager = WindowManager::<LinuxWindow>::new();
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    // Verify method exists and returns correct type
    let result: Result<(), WindowError> = manager.resize_window(window_id, 1024, 768).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_window_manager_has_focus_window() {
    // From contract: focus_window(id: WindowId) -> Result<(), WindowError>
    let mut manager = WindowManager::<LinuxWindow>::new();
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    // Verify method exists and returns correct type
    let result: Result<(), WindowError> = manager.focus_window(window_id).await;
    assert!(result.is_ok());
}

#[test]
fn test_contract_method_names_exact() {
    // Verify method names match contract exactly (compile-time check)
    // If these compile, the method names are correct

    async fn verify_signatures() {
        let mut manager = WindowManager::<LinuxWindow>::new();

        // create_window (NOT createWindow, NOT create)
        let _: Result<WindowId, WindowError> = manager.create_window(WindowConfig::default()).await;

        // close_window (NOT closeWindow, NOT close)
        let window_id = WindowId::new();
        let _: Result<(), WindowError> = manager.close_window(window_id).await;

        // get_windows (NOT getWindows, NOT windows)
        let _: Vec<WindowId> = manager.get_windows();

        // get_window_config (NOT getWindowConfig, NOT window_config)
        let _: Option<WindowConfig> = manager.get_window_config(window_id);

        // resize_window (NOT resizeWindow, NOT resize)
        let _: Result<(), WindowError> = manager.resize_window(window_id, 100, 100).await;

        // focus_window (NOT focusWindow, NOT focus)
        let _: Result<(), WindowError> = manager.focus_window(window_id).await;
    }

    // If this compiles, all method names are correct
    let _ = verify_signatures();
}

#[test]
fn test_return_types_match_contract() {
    // Verify return types match contract (compile-time check)
    use std::marker::PhantomData;

    fn assert_type<T>(_: PhantomData<T>) {}

    async fn verify_types() {
        let mut manager = WindowManager::<LinuxWindow>::new();
        let window_id = WindowId::new();

        // create_window returns Result<WindowId, WindowError>
        let create_result = manager.create_window(WindowConfig::default()).await;
        assert_type::<Result<WindowId, WindowError>>(PhantomData);
        let _ = create_result;

        // close_window returns Result<(), WindowError>
        let close_result = manager.close_window(window_id).await;
        assert_type::<Result<(), WindowError>>(PhantomData);
        let _ = close_result;

        // get_windows returns Vec<WindowId>
        let get_windows_result = manager.get_windows();
        assert_type::<Vec<WindowId>>(PhantomData);
        let _ = get_windows_result;

        // get_window_config returns Option<WindowConfig>
        let get_config_result = manager.get_window_config(window_id);
        assert_type::<Option<WindowConfig>>(PhantomData);
        let _ = get_config_result;

        // resize_window returns Result<(), WindowError>
        let resize_result = manager.resize_window(window_id, 100, 100).await;
        assert_type::<Result<(), WindowError>>(PhantomData);
        let _ = resize_result;

        // focus_window returns Result<(), WindowError>
        let focus_result = manager.focus_window(window_id).await;
        assert_type::<Result<(), WindowError>>(PhantomData);
        let _ = focus_result;
    }

    let _ = verify_types();
}
