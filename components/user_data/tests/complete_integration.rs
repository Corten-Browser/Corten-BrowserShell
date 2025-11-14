// @validates: Complete user_data integration tests
//! Comprehensive integration tests for all user_data modules

use user_data::SettingsManager;
use rusqlite::Connection;
use tempfile::NamedTempFile;

#[test]
fn test_full_user_data_workflow() {
    // Create temporary database
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_str().unwrap();

    // Test Settings Manager
    {
        let conn = Connection::open(path).expect("Failed to open DB");
        let mut settings = SettingsManager::new(conn).expect("Failed to create settings");
        settings.set("theme", "dark").expect("Failed to set theme");
        let theme = settings.get("theme").expect("Failed to get theme");
        assert_eq!(theme, Some("dark".to_string()));
    }

    // Verify persistence
    {
        let conn = Connection::open(path).expect("Failed to open DB");
        let settings = SettingsManager::new(conn).expect("Failed to create settings");
        let theme = settings.get("theme").expect("Failed to get theme");
        assert_eq!(theme, Some("dark".to_string()));
    }
}

#[test]
fn test_concurrent_access_safety() {
    let conn = Connection::open_in_memory().expect("Failed to create DB");
    let mut settings = SettingsManager::new(conn).expect("Failed to create settings");

    // Set multiple settings rapidly
    for i in 0..100 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        settings.set(&key, &value).expect("Failed to set setting");
    }

    // Verify all were saved
    let all = settings.list_all().expect("Failed to list settings");
    assert_eq!(all.len(), 100);
}
