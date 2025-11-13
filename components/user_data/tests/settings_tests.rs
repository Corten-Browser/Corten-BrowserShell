// @validates: Settings persistence, CRUD operations
//! Integration tests for Settings Manager

use user_data::settings::SettingsManager;
use rusqlite::Connection;

fn create_test_settings() -> SettingsManager {
    // Use in-memory database for tests
    let conn = Connection::open_in_memory().expect("Failed to create in-memory DB");
    SettingsManager::new(conn).expect("Failed to create settings manager")
}

#[test]
fn test_settings_manager_creation() {
    let _settings = create_test_settings();
    // Should not panic
}

#[test]
fn test_set_and_get_string_setting() {
    let mut settings = create_test_settings();

    settings.set("theme", "dark").expect("Failed to set setting");
    let value = settings.get("theme").expect("Failed to get setting");

    assert_eq!(value, Some("dark".to_string()));
}

#[test]
fn test_get_nonexistent_setting_returns_none() {
    let settings = create_test_settings();

    let value = settings.get("nonexistent").expect("Failed to get setting");

    assert_eq!(value, None);
}

#[test]
fn test_update_existing_setting() {
    let mut settings = create_test_settings();

    settings.set("language", "en").expect("Failed to set setting");
    settings.set("language", "es").expect("Failed to update setting");

    let value = settings.get("language").expect("Failed to get setting");
    assert_eq!(value, Some("es".to_string()));
}

#[test]
fn test_delete_setting() {
    let mut settings = create_test_settings();

    settings.set("temp", "value").expect("Failed to set setting");
    let deleted = settings.delete("temp").expect("Failed to delete setting");

    assert!(deleted, "Should return true when deleting existing key");

    let value = settings.get("temp").expect("Failed to get setting");
    assert_eq!(value, None, "Setting should be deleted");
}

#[test]
fn test_delete_nonexistent_setting() {
    let mut settings = create_test_settings();

    let deleted = settings.delete("nonexistent").expect("Failed to delete setting");

    assert!(!deleted, "Should return false when deleting non-existent key");
}

#[test]
fn test_list_all_settings() {
    let mut settings = create_test_settings();

    settings.set("key1", "value1").expect("Failed to set setting");
    settings.set("key2", "value2").expect("Failed to set setting");
    settings.set("key3", "value3").expect("Failed to set setting");

    let all = settings.list_all().expect("Failed to list settings");

    assert_eq!(all.len(), 3);
    assert_eq!(all.get("key1"), Some(&"value1".to_string()));
    assert_eq!(all.get("key2"), Some(&"value2".to_string()));
    assert_eq!(all.get("key3"), Some(&"value3".to_string()));
}

#[test]
fn test_clear_all_settings() {
    let mut settings = create_test_settings();

    settings.set("key1", "value1").expect("Failed to set setting");
    settings.set("key2", "value2").expect("Failed to set setting");

    settings.clear_all().expect("Failed to clear settings");

    let all = settings.list_all().expect("Failed to list settings");
    assert_eq!(all.len(), 0, "All settings should be cleared");
}

#[test]
fn test_settings_persistence_across_instances() {
    use tempfile::NamedTempFile;

    // Create temporary file for database
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_str().unwrap();

    // Create first instance and set values
    {
        let conn = Connection::open(path).expect("Failed to open DB");
        let mut settings = SettingsManager::new(conn).expect("Failed to create settings");
        settings.set("persistent", "value").expect("Failed to set setting");
    }

    // Create second instance and verify values persisted
    {
        let conn = Connection::open(path).expect("Failed to open DB");
        let settings = SettingsManager::new(conn).expect("Failed to create settings");
        let value = settings.get("persistent").expect("Failed to get setting");
        assert_eq!(value, Some("value".to_string()));
    }
}
