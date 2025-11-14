//! Unit tests for SettingsManager
//!
//! Tests get, set, save, load, and reset operations

use settings_manager::{SettingValue, SettingsManager};
use tempfile::tempdir;

#[tokio::test]
async fn test_settings_manager_new() {
    // Given: Creating a new settings manager
    // When: Initializing with defaults
    let manager = SettingsManager::new();

    // Then: It should have default settings loaded
    let all_settings = manager.get_all_settings().await.unwrap();
    assert!(
        !all_settings.is_empty(),
        "Default settings should not be empty"
    );
}

#[tokio::test]
async fn test_get_setting_exists() {
    // Given: A settings manager with defaults
    let manager = SettingsManager::new();

    // When: Getting an existing setting
    let result = manager.get_setting("window.default_width").await;

    // Then: It should return the setting value
    assert!(result.is_ok());
    match result.unwrap() {
        SettingValue::Integer(width) => assert_eq!(width, 1024),
        _ => panic!("Expected Integer variant for window width"),
    }
}

#[tokio::test]
async fn test_get_setting_not_exists() {
    // Given: A settings manager
    let manager = SettingsManager::new();

    // When: Getting a non-existent setting
    let result = manager.get_setting("non.existent.key").await;

    // Then: It should return an error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_setting_new_key() {
    // Given: A settings manager
    let manager = SettingsManager::new();

    // When: Setting a new value
    let result = manager
        .set_setting(
            "custom.setting".to_string(),
            SettingValue::String("custom_value".to_string()),
        )
        .await;

    // Then: It should succeed
    assert!(result.is_ok());

    // And: The value should be retrievable
    let retrieved = manager.get_setting("custom.setting").await.unwrap();
    match retrieved {
        SettingValue::String(s) => assert_eq!(s, "custom_value"),
        _ => panic!("Expected String variant"),
    }
}

#[tokio::test]
async fn test_set_setting_overwrite() {
    // Given: A settings manager with an existing setting
    let manager = SettingsManager::new();
    let original = manager.get_setting("window.default_width").await.unwrap();

    // When: Overwriting the setting
    manager
        .set_setting(
            "window.default_width".to_string(),
            SettingValue::Integer(1920),
        )
        .await
        .unwrap();

    // Then: The new value should be retrieved
    let new_value = manager.get_setting("window.default_width").await.unwrap();
    match new_value {
        SettingValue::Integer(width) => assert_eq!(width, 1920),
        _ => panic!("Expected Integer variant"),
    }

    // And: It should be different from the original
    match original {
        SettingValue::Integer(width) => assert_ne!(width, 1920),
        _ => panic!("Expected Integer variant"),
    }
}

#[tokio::test]
async fn test_get_all_settings() {
    // Given: A settings manager with defaults
    let manager = SettingsManager::new();

    // When: Getting all settings
    let all_settings = manager.get_all_settings().await.unwrap();

    // Then: It should contain default settings from spec
    assert!(all_settings.contains_key("window.default_width"));
    assert!(all_settings.contains_key("ui.theme"));
    assert!(all_settings.contains_key("tabs.enable_process_isolation"));
}

#[tokio::test]
async fn test_reset_to_defaults() {
    // Given: A settings manager with modified settings
    let manager = SettingsManager::new();
    manager
        .set_setting(
            "window.default_width".to_string(),
            SettingValue::Integer(1920),
        )
        .await
        .unwrap();

    // When: Resetting to defaults
    let result = manager.reset_to_defaults().await;

    // Then: It should succeed
    assert!(result.is_ok());

    // And: The setting should be back to default
    let value = manager.get_setting("window.default_width").await.unwrap();
    match value {
        SettingValue::Integer(width) => assert_eq!(width, 1024),
        _ => panic!("Expected Integer variant"),
    }
}

#[tokio::test]
async fn test_save_and_load() {
    // Given: A settings manager with custom settings
    let temp_dir = tempdir().unwrap();
    let manager = SettingsManager::with_config_dir(temp_dir.path().to_path_buf());

    manager
        .set_setting(
            "window.default_width".to_string(),
            SettingValue::Integer(1920),
        )
        .await
        .unwrap();

    manager
        .set_setting(
            "ui.theme".to_string(),
            SettingValue::String("dark".to_string()),
        )
        .await
        .unwrap();

    // When: Saving settings
    let save_result = manager.save().await;
    assert!(save_result.is_ok());

    // And: Loading into a new manager
    let new_manager = SettingsManager::with_config_dir(temp_dir.path().to_path_buf());
    let load_result = new_manager.load().await;
    assert!(load_result.is_ok());

    // Then: The settings should match
    let width = new_manager
        .get_setting("window.default_width")
        .await
        .unwrap();
    match width {
        SettingValue::Integer(w) => assert_eq!(w, 1920),
        _ => panic!("Expected Integer variant"),
    }

    let theme = new_manager.get_setting("ui.theme").await.unwrap();
    match theme {
        SettingValue::String(t) => assert_eq!(t, "dark"),
        _ => panic!("Expected String variant"),
    }
}

#[tokio::test]
async fn test_load_missing_file() {
    // Given: A settings manager with no saved file
    let temp_dir = tempdir().unwrap();
    let manager = SettingsManager::with_config_dir(temp_dir.path().to_path_buf());

    // When: Loading (no file exists)
    let result = manager.load().await;

    // Then: It should succeed (gracefully handle missing file)
    assert!(result.is_ok());

    // And: Default settings should be loaded
    let width = manager.get_setting("window.default_width").await.unwrap();
    match width {
        SettingValue::Integer(w) => assert_eq!(w, 1024),
        _ => panic!("Expected default value"),
    }
}

#[tokio::test]
async fn test_load_corrupted_file() {
    // Given: A corrupted settings file
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("settings.yaml");
    std::fs::write(&config_path, "corrupted: {invalid yaml]]]").unwrap();

    let manager = SettingsManager::with_config_dir(temp_dir.path().to_path_buf());

    // When: Loading corrupted file
    let result = manager.load().await;

    // Then: It should handle gracefully (either error or fallback to defaults)
    // Either outcome is acceptable as long as it doesn't panic
    let _ = result;

    // And: We should still be able to use defaults
    let all_settings = manager.get_all_settings().await.unwrap();
    assert!(!all_settings.is_empty());
}

#[tokio::test]
async fn test_multiple_concurrent_operations() {
    // Given: A settings manager
    let manager = SettingsManager::new();

    // When: Performing concurrent operations
    let handle1 = {
        let mgr = manager.clone();
        tokio::spawn(async move {
            mgr.set_setting("test.key1".to_string(), SettingValue::Integer(1))
                .await
        })
    };

    let handle2 = {
        let mgr = manager.clone();
        tokio::spawn(async move {
            mgr.set_setting("test.key2".to_string(), SettingValue::Integer(2))
                .await
        })
    };

    // Then: Both operations should succeed
    assert!(handle1.await.unwrap().is_ok());
    assert!(handle2.await.unwrap().is_ok());

    // And: Both values should be retrievable
    let val1 = manager.get_setting("test.key1").await.unwrap();
    let val2 = manager.get_setting("test.key2").await.unwrap();

    match (val1, val2) {
        (SettingValue::Integer(v1), SettingValue::Integer(v2)) => {
            assert_eq!(v1, 1);
            assert_eq!(v2, 2);
        }
        _ => panic!("Expected Integer variants"),
    }
}

#[tokio::test]
async fn test_default_settings_complete() {
    // Given: A new settings manager
    let manager = SettingsManager::new();
    let all_settings = manager.get_all_settings().await.unwrap();

    // Then: All required default settings should be present
    // Window settings
    assert!(all_settings.contains_key("window.default_width"));
    assert!(all_settings.contains_key("window.default_height"));
    assert!(all_settings.contains_key("window.min_width"));
    assert!(all_settings.contains_key("window.min_height"));
    assert!(all_settings.contains_key("window.allow_resize"));
    assert!(all_settings.contains_key("window.start_maximized"));

    // Tab settings
    assert!(all_settings.contains_key("tabs.enable_process_isolation"));
    assert!(all_settings.contains_key("tabs.max_processes"));
    assert!(all_settings.contains_key("tabs.recycle_after_navigations"));
    assert!(all_settings.contains_key("tabs.restore_on_crash"));
    assert!(all_settings.contains_key("tabs.lazy_loading"));

    // UI settings
    assert!(all_settings.contains_key("ui.theme"));
    assert!(all_settings.contains_key("ui.show_bookmarks_bar"));
    assert!(all_settings.contains_key("ui.show_status_bar"));
    assert!(all_settings.contains_key("ui.animations_enabled"));
    assert!(all_settings.contains_key("ui.font_size"));

    // Performance settings
    assert!(all_settings.contains_key("performance.render_fps"));
    assert!(all_settings.contains_key("performance.max_message_queue"));
    assert!(all_settings.contains_key("performance.compositor_threads"));
    assert!(all_settings.contains_key("performance.raster_threads"));

    // Security settings
    assert!(all_settings.contains_key("security.enable_sandbox"));
    assert!(all_settings.contains_key("security.allow_javascript"));
    assert!(all_settings.contains_key("security.allow_plugins"));
    assert!(all_settings.contains_key("security.block_third_party_cookies"));
    assert!(all_settings.contains_key("security.enable_webrtc"));

    // Network settings
    assert!(all_settings.contains_key("network.max_connections_per_host"));
    assert!(all_settings.contains_key("network.connection_timeout"));
    assert!(all_settings.contains_key("network.enable_http2"));
    assert!(all_settings.contains_key("network.enable_quic"));

    // Privacy settings
    assert!(all_settings.contains_key("privacy.do_not_track"));
    assert!(all_settings.contains_key("privacy.clear_on_exit"));
    assert!(all_settings.contains_key("privacy.private_browsing_available"));

    // Developer settings
    assert!(all_settings.contains_key("developer.enable_devtools"));
    assert!(all_settings.contains_key("developer.enable_extensions"));
    assert!(all_settings.contains_key("developer.allow_experimental_features"));
}
