use shared_types::*;

#[test]
fn test_window_config_creation() {
    let config = WindowConfig {
        title: "Test Window".to_string(),
        width: 1024,
        height: 768,
        x: Some(100),
        y: Some(100),
        fullscreen: false,
        resizable: true,
        decorations: true,
        always_on_top: false,
        skip_taskbar: false,
    };

    assert_eq!(config.title, "Test Window");
    assert_eq!(config.width, 1024);
    assert_eq!(config.height, 768);
    assert_eq!(config.x, Some(100));
    assert_eq!(config.y, Some(100));
    assert_eq!(config.fullscreen, false);
    assert_eq!(config.resizable, true);
    assert_eq!(config.decorations, true);
    assert_eq!(config.always_on_top, false);
    assert_eq!(config.skip_taskbar, false);
}

#[test]
fn test_window_config_default() {
    let config = WindowConfig::default();

    assert_eq!(config.title, "CortenBrowser");
    assert_eq!(config.width, 1024);
    assert_eq!(config.height, 768);
    assert_eq!(config.x, None);
    assert_eq!(config.y, None);
    assert_eq!(config.fullscreen, false);
    assert_eq!(config.resizable, true);
    assert_eq!(config.decorations, true);
    assert_eq!(config.always_on_top, false);
    assert_eq!(config.skip_taskbar, false);
}

#[test]
fn test_window_config_clone() {
    let config1 = WindowConfig {
        title: "Clone Test".to_string(),
        width: 800,
        height: 600,
        x: None,
        y: None,
        fullscreen: false,
        resizable: true,
        decorations: true,
        always_on_top: false,
        skip_taskbar: false,
    };

    let config2 = config1.clone();

    assert_eq!(config1.title, config2.title);
    assert_eq!(config1.width, config2.width);
    assert_eq!(config1.height, config2.height);
}

#[test]
fn test_window_config_serialization() {
    let config = WindowConfig {
        title: "Serialization Test".to_string(),
        width: 1920,
        height: 1080,
        x: Some(0),
        y: Some(0),
        fullscreen: true,
        resizable: false,
        decorations: false,
        always_on_top: true,
        skip_taskbar: true,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&config).expect("Failed to serialize");

    // Deserialize back
    let deserialized: WindowConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    // Should be equal
    assert_eq!(config.title, deserialized.title);
    assert_eq!(config.width, deserialized.width);
    assert_eq!(config.height, deserialized.height);
    assert_eq!(config.x, deserialized.x);
    assert_eq!(config.y, deserialized.y);
    assert_eq!(config.fullscreen, deserialized.fullscreen);
    assert_eq!(config.resizable, deserialized.resizable);
    assert_eq!(config.decorations, deserialized.decorations);
    assert_eq!(config.always_on_top, deserialized.always_on_top);
    assert_eq!(config.skip_taskbar, deserialized.skip_taskbar);
}

#[test]
fn test_window_config_with_none_position() {
    let config = WindowConfig {
        title: "No Position".to_string(),
        width: 800,
        height: 600,
        x: None,
        y: None,
        fullscreen: false,
        resizable: true,
        decorations: true,
        always_on_top: false,
        skip_taskbar: false,
    };

    assert!(config.x.is_none());
    assert!(config.y.is_none());

    // Should serialize/deserialize correctly with None values
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: WindowConfig = serde_json::from_str(&json).unwrap();

    assert!(deserialized.x.is_none());
    assert!(deserialized.y.is_none());
}
