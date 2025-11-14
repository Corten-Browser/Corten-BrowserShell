//! Integration tests with platform_abstraction
//!
//! These tests verify that window_manager correctly integrates with the
//! platform_abstraction component using real platform window implementations.

use platform_abstraction::LinuxWindow;
use shared_types::{WindowConfig, WindowError};
use window_manager::WindowManager;

#[tokio::test]
async fn test_create_window_with_linux_platform() {
    let mut manager = WindowManager::<LinuxWindow>::new();
    let config = WindowConfig {
        title: "Integration Test Window".to_string(),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let result = manager.create_window(config).await;
    assert!(
        result.is_ok(),
        "Failed to create window with Linux platform"
    );

    let window_id = result.unwrap();
    assert!(manager.get_windows().contains(&window_id));
}

#[tokio::test]
async fn test_window_lifecycle_with_platform() {
    let mut manager = WindowManager::<LinuxWindow>::new();

    // Create window
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .expect("Failed to create window");

    // Verify it exists
    assert_eq!(manager.get_windows().len(), 1);

    // Resize window
    let resize_result = manager.resize_window(window_id, 1920, 1080).await;
    assert!(
        resize_result.is_ok(),
        "Failed to resize window: {:?}",
        resize_result
    );

    // Verify resize updated config
    let config = manager.get_window_config(window_id).unwrap();
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);

    // Focus window
    let focus_result = manager.focus_window(window_id).await;
    assert!(
        focus_result.is_ok(),
        "Failed to focus window: {:?}",
        focus_result
    );

    // Close window
    let close_result = manager.close_window(window_id).await;
    assert!(
        close_result.is_ok(),
        "Failed to close window: {:?}",
        close_result
    );

    // Verify it's gone
    assert_eq!(manager.get_windows().len(), 0);
}

#[tokio::test]
async fn test_multiple_windows_with_platform() {
    let mut manager = WindowManager::<LinuxWindow>::new();

    // Create multiple windows
    let id1 = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();
    let id2 = manager
        .create_window(WindowConfig {
            title: "Window 2".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        })
        .await
        .unwrap();
    let id3 = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    // Verify all exist
    assert_eq!(manager.get_windows().len(), 3);

    // Close middle window
    manager.close_window(id2).await.unwrap();
    assert_eq!(manager.get_windows().len(), 2);

    // Verify correct windows remain
    let windows = manager.get_windows();
    assert!(windows.contains(&id1));
    assert!(!windows.contains(&id2));
    assert!(windows.contains(&id3));
}

#[tokio::test]
async fn test_window_config_persistence() {
    let mut manager = WindowManager::<LinuxWindow>::new();

    let original_config = WindowConfig {
        title: "Test Window".to_string(),
        width: 1024,
        height: 768,
        x: Some(100),
        y: Some(200),
        fullscreen: false,
        resizable: true,
        decorations: true,
        always_on_top: false,
        skip_taskbar: false,
    };

    let window_id = manager
        .create_window(original_config.clone())
        .await
        .unwrap();

    let retrieved_config = manager.get_window_config(window_id).unwrap();

    // Verify all config fields are preserved
    assert_eq!(retrieved_config.title, original_config.title);
    assert_eq!(retrieved_config.width, original_config.width);
    assert_eq!(retrieved_config.height, original_config.height);
    assert_eq!(retrieved_config.x, original_config.x);
    assert_eq!(retrieved_config.y, original_config.y);
    assert_eq!(retrieved_config.fullscreen, original_config.fullscreen);
    assert_eq!(retrieved_config.resizable, original_config.resizable);
    assert_eq!(retrieved_config.decorations, original_config.decorations);
    assert_eq!(
        retrieved_config.always_on_top,
        original_config.always_on_top
    );
    assert_eq!(retrieved_config.skip_taskbar, original_config.skip_taskbar);
}

#[tokio::test]
async fn test_error_handling_with_platform() {
    let mut manager = WindowManager::<LinuxWindow>::new();

    // Create a window
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    // Close it
    manager.close_window(window_id).await.unwrap();

    // Try to operate on closed window
    let resize_result = manager.resize_window(window_id, 800, 600).await;
    assert!(resize_result.is_err());
    match resize_result {
        Err(WindowError::NotFound(id)) => assert_eq!(id, window_id),
        _ => panic!("Expected NotFound error"),
    }

    let focus_result = manager.focus_window(window_id).await;
    assert!(focus_result.is_err());
}
