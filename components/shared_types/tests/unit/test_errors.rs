// @validates: REQ-001 (Error type definitions)
use shared_types::error::{ComponentError, TabError, WindowError};

#[test]
fn test_component_error_initialization_failed() {
    let err = ComponentError::InitializationFailed("test reason".to_string());
    assert_eq!(err.to_string(), "Initialization failed: test reason");
}

#[test]
fn test_component_error_not_initialized() {
    let err = ComponentError::NotInitialized;
    assert_eq!(err.to_string(), "Component not initialized");
}

#[test]
fn test_component_error_invalid_config() {
    let err = ComponentError::InvalidConfig("bad config".to_string());
    assert_eq!(err.to_string(), "Invalid configuration: bad config");
}

#[test]
fn test_component_error_message_error() {
    let err = ComponentError::MessageError("failed".to_string());
    assert_eq!(err.to_string(), "Message handling error: failed");
}

#[test]
fn test_component_error_shutdown_error() {
    let err = ComponentError::ShutdownError("shutdown failed".to_string());
    assert_eq!(err.to_string(), "Component shutdown error: shutdown failed");
}

#[test]
fn test_component_error_internal() {
    let err = ComponentError::Internal("internal error".to_string());
    assert_eq!(err.to_string(), "Internal error: internal error");
}

#[test]
fn test_tab_error_not_found() {
    use shared_types::tab::TabId;
    let tab_id = TabId(12345);
    let err = TabError::NotFound(tab_id);
    assert!(err.to_string().contains("Tab not found"));
}

#[test]
fn test_tab_error_creation_failed() {
    let err = TabError::CreationFailed("reason".to_string());
    assert_eq!(err.to_string(), "Tab creation failed: reason");
}

#[test]
fn test_tab_error_invalid_url() {
    let err = TabError::InvalidUrl("bad url".to_string());
    assert_eq!(err.to_string(), "Invalid URL: bad url");
}

#[test]
fn test_tab_error_navigation_failed() {
    let err = TabError::NavigationFailed("nav failed".to_string());
    assert_eq!(err.to_string(), "Navigation failed: nav failed");
}

#[test]
fn test_window_error_not_found() {
    use shared_types::window::WindowId;
    let window_id = WindowId(67890);
    let err = WindowError::NotFound(window_id);
    assert!(err.to_string().contains("Window not found"));
}

#[test]
fn test_window_error_creation_failed() {
    let err = WindowError::CreationFailed("failed".to_string());
    assert_eq!(err.to_string(), "Window creation failed: failed");
}

#[test]
fn test_window_error_invalid_config() {
    let err = WindowError::InvalidConfig("invalid".to_string());
    assert_eq!(err.to_string(), "Invalid window configuration: invalid");
}

#[test]
fn test_window_error_platform_error() {
    let err = WindowError::PlatformError("platform issue".to_string());
    assert_eq!(err.to_string(), "Platform error: platform issue");
}
