//! Unit tests for Window struct

use platform_abstraction::{PlatformHandle, PlatformWindow};
use shared_types::{TabId, WindowConfig, WindowError};
use window_manager::Window;

// Mock platform window for testing
struct MockPlatformWindow {
    handle: PlatformHandle,
}

impl PlatformWindow for MockPlatformWindow {
    fn create(_config: &WindowConfig) -> Result<Self, WindowError> {
        Ok(Self {
            handle: PlatformHandle::Linux(platform_abstraction::LinuxHandle { window: 12345 }),
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

#[test]
fn test_window_has_id() {
    let config = WindowConfig::default();
    let platform_window = MockPlatformWindow::create(&config).unwrap();
    let window = Window::new(config.clone(), platform_window);

    // Window should have a valid ID
    assert_ne!(window.id.as_uuid().as_u128(), 0);
}

#[test]
fn test_window_stores_config() {
    let config = WindowConfig {
        title: "Test Window".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    let platform_window = MockPlatformWindow::create(&config).unwrap();
    let window = Window::new(config.clone(), platform_window);

    assert_eq!(window.config.title, "Test Window");
    assert_eq!(window.config.width, 800);
    assert_eq!(window.config.height, 600);
}

#[test]
fn test_window_starts_with_no_tabs() {
    let config = WindowConfig::default();
    let platform_window = MockPlatformWindow::create(&config).unwrap();
    let window = Window::new(config, platform_window);

    assert_eq!(window.tabs.len(), 0);
    assert_eq!(window.active_tab, None);
}

#[test]
fn test_window_can_add_tab() {
    let config = WindowConfig::default();
    let platform_window = MockPlatformWindow::create(&config).unwrap();
    let mut window = Window::new(config, platform_window);

    let tab_id = TabId::new();
    window.add_tab(tab_id);

    assert_eq!(window.tabs.len(), 1);
    assert!(window.tabs.contains(&tab_id));
}

#[test]
fn test_window_can_remove_tab() {
    let config = WindowConfig::default();
    let platform_window = MockPlatformWindow::create(&config).unwrap();
    let mut window = Window::new(config, platform_window);

    let tab_id = TabId::new();
    window.add_tab(tab_id);
    assert_eq!(window.tabs.len(), 1);

    window.remove_tab(&tab_id);
    assert_eq!(window.tabs.len(), 0);
}

#[test]
fn test_window_tracks_active_tab() {
    let config = WindowConfig::default();
    let platform_window = MockPlatformWindow::create(&config).unwrap();
    let mut window = Window::new(config, platform_window);

    let tab_id = TabId::new();
    window.add_tab(tab_id);
    window.set_active_tab(Some(tab_id));

    assert_eq!(window.active_tab, Some(tab_id));
}

#[test]
fn test_window_has_platform_handle() {
    let config = WindowConfig::default();
    let platform_window = MockPlatformWindow::create(&config).unwrap();
    let window = Window::new(config, platform_window);

    match window.platform_handle {
        PlatformHandle::Linux(handle) => assert_eq!(handle.window, 12345),
        _ => panic!("Expected Linux handle"),
    }
}
