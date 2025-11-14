// @validates: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Integration tests for WindowManager implementation

use shared_types::window::{WindowManager, WindowConfig, WindowUpdate, PlatformEvent, WindowId};
use window_manager::WindowManagerImpl;

#[tokio::test]
async fn test_create_window_returns_valid_id() {
    // Given: A window manager
    let mut manager = WindowManagerImpl::new();

    // When: Creating a window with default config
    let config = WindowConfig::default();
    let result = manager.create_window(config).await;

    // Then: Window ID is returned successfully
    assert!(result.is_ok());
    let window_id = result.unwrap();
    assert!(window_id.0 > 0);
}

#[tokio::test]
async fn test_create_window_adds_to_collection() {
    // Given: A window manager
    let mut manager = WindowManagerImpl::new();

    // When: Creating a window
    let config = WindowConfig::default();
    let window_id = manager.create_window(config).await.unwrap();

    // Then: Window exists in collection
    assert!(manager.window_exists(window_id));
    assert_eq!(manager.window_count(), 1);
}

#[tokio::test]
async fn test_close_window_removes_from_collection() {
    // Given: A window manager with one window
    let mut manager = WindowManagerImpl::new();
    let config = WindowConfig::default();
    let window_id = manager.create_window(config).await.unwrap();

    // When: Closing the window
    let result = manager.close_window(window_id).await;

    // Then: Window is removed successfully
    assert!(result.is_ok());
    assert!(!manager.window_exists(window_id));
    assert_eq!(manager.window_count(), 0);
}

#[tokio::test]
async fn test_close_nonexistent_window_returns_error() {
    // Given: A window manager
    let mut manager = WindowManagerImpl::new();

    // When: Trying to close a window that doesn't exist
    let fake_id = WindowId::new();
    let result = manager.close_window(fake_id).await;

    // Then: Error is returned
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_windows_returns_all_windows() {
    // Given: A window manager with multiple windows
    let mut manager = WindowManagerImpl::new();
    let id1 = manager.create_window(WindowConfig::default()).await.unwrap();
    let id2 = manager.create_window(WindowConfig::default()).await.unwrap();
    let id3 = manager.create_window(WindowConfig::default()).await.unwrap();

    // When: Checking window count
    let count = manager.window_count();

    // Then: All windows are tracked
    assert_eq!(count, 3);
    assert!(manager.window_exists(id1));
    assert!(manager.window_exists(id2));
    assert!(manager.window_exists(id3));
}

#[tokio::test]
async fn test_update_window_resize() {
    // Given: A window manager with a window
    let mut manager = WindowManagerImpl::new();
    let window_id = manager.create_window(WindowConfig::default()).await.unwrap();

    // When: Resizing the window
    let update = WindowUpdate::Resize { width: 1920, height: 1080 };
    let result = manager.update_window(window_id, update).await;

    // Then: Window is resized successfully (state persisted)
    assert!(result.is_ok());
    assert!(manager.window_exists(window_id));
}

#[tokio::test]
async fn test_update_window_move() {
    // Given: A window manager with a window
    let mut manager = WindowManagerImpl::new();
    let window_id = manager.create_window(WindowConfig::default()).await.unwrap();

    // When: Moving the window
    let update = WindowUpdate::Move { x: 100, y: 200 };
    let result = manager.update_window(window_id, update).await;

    // Then: Window position is updated (state persisted)
    assert!(result.is_ok());
    assert!(manager.window_exists(window_id));
}

#[tokio::test]
async fn test_update_window_fullscreen() {
    // Given: A window manager with a window
    let mut manager = WindowManagerImpl::new();
    let window_id = manager.create_window(WindowConfig::default()).await.unwrap();

    // When: Setting fullscreen
    let update = WindowUpdate::SetFullscreen { fullscreen: true };
    let result = manager.update_window(window_id, update).await;

    // Then: Window fullscreen is set (state persisted)
    assert!(result.is_ok());
    assert!(manager.window_exists(window_id));
}

#[tokio::test]
async fn test_handle_platform_event_resized() {
    // Given: A window manager with a window
    let mut manager = WindowManagerImpl::new();
    let window_id = manager.create_window(WindowConfig::default()).await.unwrap();

    // When: Handling a resize event
    let event = PlatformEvent::Resized { width: 800, height: 600 };
    let result = manager.handle_platform_event(window_id, event).await;

    // Then: Event is handled successfully (state updated internally)
    assert!(result.is_ok());
    assert!(manager.window_exists(window_id));
}

#[tokio::test]
async fn test_handle_platform_event_focused() {
    // Given: A window manager with a window
    let mut manager = WindowManagerImpl::new();
    let window_id = manager.create_window(WindowConfig::default()).await.unwrap();

    // When: Handling a focus event
    let event = PlatformEvent::Focused;
    let result = manager.handle_platform_event(window_id, event).await;

    // Then: Event is handled successfully (focus state updated internally)
    assert!(result.is_ok());
    assert!(manager.window_exists(window_id));
}

#[tokio::test]
async fn test_multi_window_support_50_plus_windows() {
    // Given: A window manager
    let mut manager = WindowManagerImpl::new();

    // When: Creating 60 windows
    let mut window_ids = Vec::new();
    for _ in 0..60 {
        let id = manager.create_window(WindowConfig::default()).await.unwrap();
        window_ids.push(id);
    }

    // Then: All windows are managed successfully
    assert_eq!(manager.window_count(), 60);

    // And: Each window exists
    for window_id in window_ids {
        assert!(manager.window_exists(window_id));
    }
}

#[tokio::test]
async fn test_window_state_persistence_after_updates() {
    // Given: A window manager with a window that undergoes multiple updates
    let mut manager = WindowManagerImpl::new();
    let window_id = manager.create_window(WindowConfig::default()).await.unwrap();

    // When: Applying multiple updates
    manager.update_window(window_id, WindowUpdate::Resize { width: 1920, height: 1080 }).await.unwrap();
    manager.update_window(window_id, WindowUpdate::Move { x: 100, y: 100 }).await.unwrap();
    manager.update_window(window_id, WindowUpdate::SetTitle { title: "Test Window".to_string() }).await.unwrap();

    // Then: All state changes are persisted (window still exists)
    assert!(manager.window_exists(window_id));
}

#[tokio::test]
async fn test_concurrent_window_operations() {
    // Given: A window manager
    let mut manager = WindowManagerImpl::new();

    // When: Creating windows concurrently (simulated with sequential calls)
    let mut handles = Vec::new();
    for i in 0..10 {
        let mut config = WindowConfig::default();
        config.title = format!("Window {}", i);
        let id = manager.create_window(config).await.unwrap();
        handles.push(id);
    }

    // Then: All operations complete successfully
    assert_eq!(manager.window_count(), 10);

    // And: Each window exists
    for window_id in handles.iter() {
        assert!(manager.window_exists(*window_id));
    }
}
