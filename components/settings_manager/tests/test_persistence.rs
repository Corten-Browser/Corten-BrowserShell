//! Integration tests for settings persistence
//!
//! Tests real file system operations for save/load functionality

use settings_manager::{SettingValue, SettingsManager};
use tempfile::tempdir;

#[tokio::test]
async fn test_persistence_across_restarts() {
    // Given: A settings manager with custom settings
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    {
        let manager = SettingsManager::with_config_dir(config_dir.clone());

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

        manager
            .set_setting(
                "performance.render_fps".to_string(),
                SettingValue::Integer(144),
            )
            .await
            .unwrap();

        // When: Saving settings
        manager.save().await.unwrap();
    } // manager drops here

    // And: Creating a new manager (simulating restart)
    let new_manager = SettingsManager::with_config_dir(config_dir.clone());
    new_manager.load().await.unwrap();

    // Then: All settings should be persisted
    let width = new_manager
        .get_setting("window.default_width")
        .await
        .unwrap();
    match width {
        SettingValue::Integer(w) => assert_eq!(w, 1920),
        _ => panic!("Expected Integer"),
    }

    let theme = new_manager.get_setting("ui.theme").await.unwrap();
    match theme {
        SettingValue::String(t) => assert_eq!(t, "dark"),
        _ => panic!("Expected String"),
    }

    let fps = new_manager
        .get_setting("performance.render_fps")
        .await
        .unwrap();
    match fps {
        SettingValue::Integer(f) => assert_eq!(f, 144),
        _ => panic!("Expected Integer"),
    }
}

#[tokio::test]
async fn test_file_format_is_yaml() {
    // Given: A settings manager
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    let manager = SettingsManager::with_config_dir(config_dir.clone());

    manager
        .set_setting(
            "test.key".to_string(),
            SettingValue::String("test_value".to_string()),
        )
        .await
        .unwrap();

    // When: Saving
    manager.save().await.unwrap();

    // Then: The file should be valid YAML
    let config_file = config_dir.join("settings.yaml");
    assert!(config_file.exists());

    let content = std::fs::read_to_string(&config_file).unwrap();

    // Verify it's valid YAML by parsing
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
    assert!(parsed.is_mapping());
}

#[tokio::test]
async fn test_multiple_save_load_cycles() {
    // Given: A settings manager
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    let manager = SettingsManager::with_config_dir(config_dir.clone());

    // When: Performing multiple save/load cycles
    for i in 0..5 {
        manager
            .set_setting("test.counter".to_string(), SettingValue::Integer(i))
            .await
            .unwrap();

        manager.save().await.unwrap();
        manager.load().await.unwrap();

        let value = manager.get_setting("test.counter").await.unwrap();
        match value {
            SettingValue::Integer(v) => assert_eq!(v, i),
            _ => panic!("Expected Integer"),
        }
    }

    // Then: The final value should be correct
    let final_value = manager.get_setting("test.counter").await.unwrap();
    match final_value {
        SettingValue::Integer(v) => assert_eq!(v, 4),
        _ => panic!("Expected Integer"),
    }
}

#[tokio::test]
async fn test_handles_missing_config_directory() {
    // Given: A non-existent config directory
    let temp_dir = tempdir().unwrap();
    let non_existent = temp_dir.path().join("non_existent");

    // When: Creating manager with non-existent directory
    let manager = SettingsManager::with_config_dir(non_existent.clone());

    // And: Saving settings
    manager
        .set_setting(
            "test.key".to_string(),
            SettingValue::String("test".to_string()),
        )
        .await
        .unwrap();

    let result = manager.save().await;

    // Then: It should create the directory and succeed
    assert!(result.is_ok());
    assert!(non_existent.exists());
}

#[tokio::test]
async fn test_partial_settings_merge_with_defaults() {
    // Given: A settings file with partial settings
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    let config_file = config_dir.join("settings.yaml");

    std::fs::create_dir_all(&config_dir).unwrap();

    // Write partial settings (only a few keys)
    let partial_yaml = r#"
window.default_width: !Integer 800
ui.theme: !String "custom"
"#;
    std::fs::write(&config_file, partial_yaml).unwrap();

    // When: Loading
    let manager = SettingsManager::with_config_dir(config_dir);
    manager.load().await.unwrap();

    // Then: Loaded settings should exist
    let width = manager.get_setting("window.default_width").await.unwrap();
    match width {
        SettingValue::Integer(w) => assert_eq!(w, 800),
        _ => panic!("Expected Integer"),
    }

    // And: Default settings should still exist
    let height = manager.get_setting("window.default_height").await.unwrap();
    match height {
        SettingValue::Integer(h) => assert_eq!(h, 768), // default value
        _ => panic!("Expected Integer"),
    }
}

#[tokio::test]
async fn test_concurrent_save_operations() {
    // Given: A settings manager
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    let manager = SettingsManager::with_config_dir(config_dir.clone());

    // When: Performing concurrent saves
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let mgr = manager.clone();
            tokio::spawn(async move {
                mgr.set_setting(format!("test.key{}", i), SettingValue::Integer(i))
                    .await
                    .unwrap();
                mgr.save().await
            })
        })
        .collect();

    // Then: All saves should succeed
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // And: All values should be retrievable
    let new_manager = SettingsManager::with_config_dir(config_dir);
    new_manager.load().await.unwrap();

    for i in 0..10 {
        let value = new_manager
            .get_setting(&format!("test.key{}", i))
            .await
            .unwrap();
        match value {
            SettingValue::Integer(v) => assert_eq!(v, i),
            _ => panic!("Expected Integer"),
        }
    }
}
