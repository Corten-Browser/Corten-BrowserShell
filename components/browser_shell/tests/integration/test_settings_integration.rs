// @validates: REQ-006
//! Integration Test: Settings Integration
//!
//! Verifies that settings can be stored and retrieved through the browser_shell API,
//! integrating with the user_data component's SettingsManager.

use browser_shell::BrowserShell;

#[tokio::test]
async fn test_set_and_get_setting() {
    // Given: A running BrowserShell instance
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Setting a configuration value
    let set_result = api
        .set_setting("homepage", "https://example.com".to_string())
        .await;

    // Then: Setting should be stored successfully
    assert!(
        set_result.is_ok(),
        "Setting should be stored: {:?}",
        set_result.err()
    );

    // When: Retrieving the same setting
    let get_result = api.get_setting("homepage").await;

    // Then: Setting should be retrieved with correct value
    assert!(get_result.is_ok(), "Setting should be retrieved");
    assert_eq!(
        get_result.unwrap(),
        "https://example.com",
        "Retrieved setting should match stored value"
    );
}

#[tokio::test]
async fn test_set_multiple_settings() {
    // Given: A running BrowserShell
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Setting multiple configuration values
    api.set_setting("theme", "dark".to_string())
        .await
        .expect("Should set theme");

    api.set_setting("language", "en-US".to_string())
        .await
        .expect("Should set language");

    api.set_setting("zoom_level", "1.2".to_string())
        .await
        .expect("Should set zoom level");

    // Then: All settings should be retrievable
    assert_eq!(
        api.get_setting("theme").await.unwrap(),
        "dark",
        "Theme should be retrievable"
    );

    assert_eq!(
        api.get_setting("language").await.unwrap(),
        "en-US",
        "Language should be retrievable"
    );

    assert_eq!(
        api.get_setting("zoom_level").await.unwrap(),
        "1.2",
        "Zoom level should be retrievable"
    );
}

#[tokio::test]
async fn test_update_existing_setting() {
    // Given: A setting exists with an initial value
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    api.set_setting("homepage", "https://example.com".to_string())
        .await
        .expect("Initial setting should be stored");

    // When: Updating the setting to a new value
    api.set_setting("homepage", "https://new-homepage.com".to_string())
        .await
        .expect("Setting should be updated");

    // Then: Retrieved value should reflect the update
    assert_eq!(
        api.get_setting("homepage").await.unwrap(),
        "https://new-homepage.com",
        "Setting should be updated to new value"
    );
}

#[tokio::test]
async fn test_settings_persist_across_operations() {
    // Given: Settings are stored
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    api.set_setting("user_preference", "value1".to_string())
        .await
        .expect("Setting should be stored");

    // When: Performing other operations (like creating windows)
    let window_id = api
        .new_window(shared_types::WindowConfig::default())
        .await
        .expect("Window should be created");

    // Then: Settings should still be accessible
    assert_eq!(
        api.get_setting("user_preference").await.unwrap(),
        "value1",
        "Settings should persist across other operations"
    );

    // Cleanup
    api.close_window(window_id).await.ok();
}

#[tokio::test]
async fn test_get_nonexistent_setting() {
    // Given: A running BrowserShell
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Trying to get a setting that doesn't exist
    let result = api.get_setting("nonexistent_key").await;

    // Then: Should return an error
    assert!(
        result.is_err(),
        "Getting nonexistent setting should return error"
    );
}

#[tokio::test]
async fn test_settings_manager_integration() {
    // Given: BrowserShell initialized (which initializes SettingsManager)
    let browser = BrowserShell::new()
        .await
        .expect("BrowserShell should initialize");

    let api = browser.api();

    // When: Using settings functionality (which uses user_data::SettingsManager)
    api.set_setting("test_key", "test_value".to_string())
        .await
        .expect("SettingsManager should be initialized and functional");

    // Then: Settings should work correctly
    let value = api
        .get_setting("test_key")
        .await
        .expect("SettingsManager should retrieve values");

    assert_eq!(
        value, "test_value",
        "SettingsManager integration should work correctly"
    );
}
