use shared_types::*;
use std::error::Error;

// ComponentError tests
#[test]
fn test_component_error_variants() {
    let errors = vec![
        ComponentError::InitializationFailed("test".to_string()),
        ComponentError::MessageRoutingFailed("test".to_string()),
        ComponentError::InvalidState("test".to_string()),
        ComponentError::ResourceNotFound("test".to_string()),
        ComponentError::PermissionDenied("test".to_string()),
    ];

    assert_eq!(errors.len(), 5);
}

#[test]
fn test_component_error_implements_error_trait() {
    let error = ComponentError::InitializationFailed("test error".to_string());

    // Should implement Error trait
    let _error_trait: &dyn Error = &error;

    // Should have a Display implementation (via Error trait)
    let display_str = format!("{}", error);
    assert!(!display_str.is_empty());
}

#[test]
fn test_component_error_display() {
    let error = ComponentError::InitializationFailed("Component X failed to init".to_string());
    let display = format!("{}", error);

    assert!(display.contains("Component X failed to init"));
}

#[test]
fn test_component_error_debug() {
    let error = ComponentError::MessageRoutingFailed("routing error".to_string());
    let debug = format!("{:?}", error);

    assert!(debug.contains("MessageRoutingFailed"));
}

#[test]
fn test_component_error_serialization() {
    let error = ComponentError::InvalidState("invalid state".to_string());

    let json = serde_json::to_string(&error).expect("Failed to serialize");
    let deserialized: ComponentError = serde_json::from_str(&json).expect("Failed to deserialize");

    // Compare display strings since we can't derive PartialEq for errors
    assert_eq!(format!("{:?}", error), format!("{:?}", deserialized));
}

// WindowError tests
#[test]
fn test_window_error_variants() {
    let window_id = WindowId::new();

    let errors = vec![
        WindowError::CreationFailed("test".to_string()),
        WindowError::NotFound(window_id),
        WindowError::InvalidConfig("test".to_string()),
        WindowError::PlatformError("test".to_string()),
    ];

    assert_eq!(errors.len(), 4);
}

#[test]
fn test_window_error_implements_error_trait() {
    let error = WindowError::CreationFailed("window creation failed".to_string());

    // Should implement Error trait
    let _error_trait: &dyn Error = &error;

    // Should have a Display implementation
    let display_str = format!("{}", error);
    assert!(!display_str.is_empty());
}

#[test]
fn test_window_error_display() {
    let error = WindowError::CreationFailed("Failed to create window".to_string());
    let display = format!("{}", error);

    assert!(display.contains("Failed to create window"));
}

#[test]
fn test_window_error_not_found() {
    let window_id = WindowId::new();
    let error = WindowError::NotFound(window_id);
    let display = format!("{}", error);

    assert!(display.contains("not found") || display.contains("NotFound"));
}

#[test]
fn test_window_error_serialization() {
    let error = WindowError::InvalidConfig("bad config".to_string());

    let json = serde_json::to_string(&error).expect("Failed to serialize");
    let deserialized: WindowError = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(format!("{:?}", error), format!("{:?}", deserialized));
}

// TabError tests
#[test]
fn test_tab_error_variants() {
    let tab_id = TabId::new();

    let errors = vec![
        TabError::CreationFailed("test".to_string()),
        TabError::NotFound(tab_id),
        TabError::NavigationFailed("test".to_string()),
        TabError::ProcessIsolationFailed("test".to_string()),
    ];

    assert_eq!(errors.len(), 4);
}

#[test]
fn test_tab_error_implements_error_trait() {
    let error = TabError::CreationFailed("tab creation failed".to_string());

    // Should implement Error trait
    let _error_trait: &dyn Error = &error;

    // Should have a Display implementation
    let display_str = format!("{}", error);
    assert!(!display_str.is_empty());
}

#[test]
fn test_tab_error_display() {
    let error = TabError::NavigationFailed("https://example.com".to_string());
    let display = format!("{}", error);

    assert!(display.contains("example.com") || display.contains("NavigationFailed"));
}

#[test]
fn test_tab_error_not_found() {
    let tab_id = TabId::new();
    let error = TabError::NotFound(tab_id);
    let display = format!("{}", error);

    assert!(display.contains("not found") || display.contains("NotFound"));
}

#[test]
fn test_tab_error_serialization() {
    let error = TabError::ProcessIsolationFailed("isolation error".to_string());

    let json = serde_json::to_string(&error).expect("Failed to serialize");
    let deserialized: TabError = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(format!("{:?}", error), format!("{:?}", deserialized));
}

#[test]
fn test_all_errors_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<ComponentError>();
    assert_send_sync::<WindowError>();
    assert_send_sync::<TabError>();
}
