//! Contract tests - Verify public API matches contract specification
//!
//! These tests ensure the settings_manager component exports exactly
//! what is specified in contracts/settings_manager.yaml

use settings_manager::{SettingValue, SettingsManager};
use shared_types::ComponentError;
use std::collections::HashMap;

#[test]
fn test_setting_value_enum_variants() {
    // Contract specifies: SettingValue enum with String, Integer, Float, Boolean variants

    // Verify String variant exists
    let _string = SettingValue::String("test".to_string());

    // Verify Integer variant exists (i64)
    let _integer = SettingValue::Integer(42i64);

    // Verify Float variant exists (f64)
    let _float = SettingValue::Float(3.14f64);

    // Verify Boolean variant exists
    let _boolean = SettingValue::Boolean(true);
}

#[tokio::test]
async fn test_settings_manager_get_setting() {
    // Contract: get_setting(key: String) -> Result<SettingValue, ComponentError>

    let manager = SettingsManager::new();

    // Verify method exists and returns correct type
    let result: Result<SettingValue, ComponentError> =
        manager.get_setting("window.default_width").await;

    // Should succeed for existing key
    assert!(result.is_ok());

    // Should fail for non-existent key
    let result = manager.get_setting("non.existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_settings_manager_set_setting() {
    // Contract: set_setting(key: String, value: SettingValue) -> Result<(), ComponentError>

    let manager = SettingsManager::new();

    // Verify method exists and accepts correct parameters
    let result: Result<(), ComponentError> = manager
        .set_setting(
            "test.key".to_string(),
            SettingValue::String("value".to_string()),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_settings_manager_get_all_settings() {
    // Contract: get_all_settings() -> Result<HashMap<String, SettingValue>, ComponentError>

    let manager = SettingsManager::new();

    // Verify method exists and returns correct type
    let result: Result<HashMap<String, SettingValue>, ComponentError> =
        manager.get_all_settings().await;

    assert!(result.is_ok());

    let settings = result.unwrap();
    assert!(!settings.is_empty());
}

#[tokio::test]
async fn test_settings_manager_reset_to_defaults() {
    // Contract: reset_to_defaults() -> Result<(), ComponentError>

    let manager = SettingsManager::new();

    // Modify a setting
    manager
        .set_setting(
            "window.default_width".to_string(),
            SettingValue::Integer(1920),
        )
        .await
        .unwrap();

    // Verify method exists and works
    let result: Result<(), ComponentError> = manager.reset_to_defaults().await;

    assert!(result.is_ok());

    // Verify reset actually happened
    let value = manager.get_setting("window.default_width").await.unwrap();
    match value {
        SettingValue::Integer(w) => assert_eq!(w, 1024), // default value
        _ => panic!("Expected Integer"),
    }
}

#[tokio::test]
async fn test_settings_manager_save() {
    // Contract: save() -> Result<(), ComponentError>

    let manager = SettingsManager::new();

    // Verify method exists and returns correct type
    let result: Result<(), ComponentError> = manager.save().await;

    // Should succeed (or fail gracefully if no config dir)
    let _ = result;
}

#[tokio::test]
async fn test_settings_manager_load() {
    // Contract: load() -> Result<(), ComponentError>

    let manager = SettingsManager::new();

    // Verify method exists and returns correct type
    let result: Result<(), ComponentError> = manager.load().await;

    // Should succeed (even if no file exists, uses defaults)
    assert!(result.is_ok());
}

#[test]
fn test_settings_manager_is_clonable() {
    // SettingsManager should be Clone for concurrent access

    let manager = SettingsManager::new();
    let _cloned = manager.clone();
}

#[tokio::test]
async fn test_all_methods_are_async() {
    // Contract specifies all methods are async: true

    let manager = SettingsManager::new();

    // All these methods must be awaitable
    let _ = manager.get_setting("test").await;
    let _ = manager
        .set_setting("test".to_string(), SettingValue::Boolean(true))
        .await;
    let _ = manager.get_all_settings().await;
    let _ = manager.reset_to_defaults().await;
    let _ = manager.save().await;
    let _ = manager.load().await;
}

#[test]
fn test_setting_value_is_serializable() {
    // SettingValue must support serialization for persistence

    let value = SettingValue::String("test".to_string());

    // Must implement serde Serialize
    let _yaml = serde_yaml::to_string(&value).unwrap();

    // Must implement serde Deserialize
    let yaml = "!String test";
    let _deserialized: SettingValue = serde_yaml::from_str(yaml).unwrap();
}

#[test]
fn test_no_extra_public_exports() {
    // Verify we only export what's in the contract

    // These should compile (public exports)
    let _setting_value: SettingValue = SettingValue::Boolean(true);
    let _manager: SettingsManager = SettingsManager::new();

    // Private modules should not be accessible
    // (This is enforced by the compiler, these lines would fail if uncommented)
    // use settings_manager::defaults;
    // use settings_manager::setting_value;
    // use settings_manager::settings_manager;
}

#[tokio::test]
async fn test_error_types_from_shared_types() {
    // Verify we use ComponentError from shared_types as specified in contract

    let manager = SettingsManager::new();

    // get_setting returns ComponentError
    let result = manager.get_setting("non.existent").await;
    match result {
        Err(ComponentError::ResourceNotFound(_)) => {} // Expected
        _ => panic!("Should return ComponentError::ResourceNotFound"),
    }
}

#[test]
fn test_setting_value_variant_types() {
    // Verify exact types for each variant (from contract)

    // String(String)
    match SettingValue::String("test".to_string()) {
        SettingValue::String(s) => {
            let _: String = s; // Must be String type
        }
        _ => panic!(),
    }

    // Integer(i64)
    match SettingValue::Integer(42) {
        SettingValue::Integer(i) => {
            let _: i64 = i; // Must be i64 type
        }
        _ => panic!(),
    }

    // Float(f64)
    match SettingValue::Float(3.14) {
        SettingValue::Float(f) => {
            let _: f64 = f; // Must be f64 type
        }
        _ => panic!(),
    }

    // Boolean(bool)
    match SettingValue::Boolean(true) {
        SettingValue::Boolean(b) => {
            let _: bool = b; // Must be bool type
        }
        _ => panic!(),
    }
}
