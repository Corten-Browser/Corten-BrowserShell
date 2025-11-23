//! Unit tests for WindowManager

use platform_abstraction::{PlatformHandle, PlatformWindow};
use shared_types::{WindowConfig, WindowError, WindowId};
use window_manager::WindowManager;

// Mock platform window for testing
struct MockPlatformWindow {
    handle: PlatformHandle,
}

impl PlatformWindow for MockPlatformWindow {
    fn create(_config: &WindowConfig) -> Result<Self, WindowError> {
        Ok(Self {
            handle: PlatformHandle::LinuxX11(platform_abstraction::LinuxX11Handle { window: 12345, display: 0, screen: 0, visual_id: 0 }),
        })
    }

    fn destroy(&mut self) -> Result<(), WindowError> {
        Ok(())
    }

    fn show(&mut self) -> Result<(), WindowError> {
        Ok(())
    }

    fn hide(&mut self) -> Result<(), WindowError> {
        Ok(())
    }

    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), WindowError> {
        Ok(())
    }

    fn move_to(&mut self, _x: i32, _y: i32) -> Result<(), WindowError> {
        Ok(())
    }

    fn focus(&mut self) -> Result<(), WindowError> {
        Ok(())
    }

    fn get_handle(&self) -> PlatformHandle {
        self.handle.clone()
    }
}

#[tokio::test]
async fn test_window_manager_starts_empty() {
    let manager = WindowManager::<MockPlatformWindow>::new();
    assert_eq!(manager.get_windows().len(), 0);
}

#[tokio::test]
async fn test_create_window() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let config = WindowConfig::default();

    let result = manager.create_window(config.clone()).await;
    assert!(result.is_ok());

    let window_id = result.unwrap();
    assert_eq!(manager.get_windows().len(), 1);
    assert!(manager.get_windows().contains(&window_id));
}

#[tokio::test]
async fn test_create_multiple_windows() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();

    let id1 = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();
    let id2 = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    assert_ne!(id1, id2);
    assert_eq!(manager.get_windows().len(), 2);
}

#[tokio::test]
async fn test_close_window() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    let result = manager.close_window(window_id).await;
    assert!(result.is_ok());
    assert_eq!(manager.get_windows().len(), 0);
}

#[tokio::test]
async fn test_close_nonexistent_window() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let fake_id = WindowId::new();

    let result = manager.close_window(fake_id).await;
    assert!(result.is_err());
    match result {
        Err(WindowError::NotFound(id)) => assert_eq!(id, fake_id),
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_get_window_config() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let config = WindowConfig {
        title: "Test".to_string(),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let window_id = manager.create_window(config.clone()).await.unwrap();
    let retrieved_config = manager.get_window_config(window_id);

    assert!(retrieved_config.is_some());
    let retrieved_config = retrieved_config.unwrap();
    assert_eq!(retrieved_config.title, "Test");
    assert_eq!(retrieved_config.width, 1024);
    assert_eq!(retrieved_config.height, 768);
}

#[tokio::test]
async fn test_get_config_nonexistent_window() {
    let manager = WindowManager::<MockPlatformWindow>::new();
    let fake_id = WindowId::new();

    let config = manager.get_window_config(fake_id);
    assert!(config.is_none());
}

#[tokio::test]
async fn test_resize_window() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    let result = manager.resize_window(window_id, 1920, 1080).await;
    assert!(result.is_ok());

    // Verify window config was updated
    let config = manager.get_window_config(window_id).unwrap();
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
}

#[tokio::test]
async fn test_resize_nonexistent_window() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let fake_id = WindowId::new();

    let result = manager.resize_window(fake_id, 1920, 1080).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_focus_window() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let window_id = manager
        .create_window(WindowConfig::default())
        .await
        .unwrap();

    let result = manager.focus_window(window_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_focus_nonexistent_window() {
    let mut manager = WindowManager::<MockPlatformWindow>::new();
    let fake_id = WindowId::new();

    let result = manager.focus_window(fake_id).await;
    assert!(result.is_err());
}
