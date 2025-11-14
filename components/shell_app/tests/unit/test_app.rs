//! Tests for BrowserApp initialization and GUI launch

use shell_app::{AppConfig, BrowserApp};
use tempfile::TempDir;

#[tokio::test]
async fn test_browser_app_new() {
    // Given: An AppConfig
    let temp_dir = TempDir::new().unwrap();
    let config = AppConfig {
        user_data_dir: Some(temp_dir.path().to_str().unwrap().to_string()),
        initial_url: Some("https://example.com".to_string()),
        fullscreen: false,
        headless: true,
        enable_devtools: false,
        log_level: shell_app::LogLevel::Info,
    };

    // When: Creating a new BrowserApp
    let app = BrowserApp::new(config).await;

    // Then: BrowserApp should be created successfully
    assert!(app.is_ok());
}

#[tokio::test]
async fn test_browser_app_initialization() {
    // Given: A headless config
    let temp_dir = TempDir::new().unwrap();
    let config = AppConfig {
        user_data_dir: Some(temp_dir.path().to_str().unwrap().to_string()),
        initial_url: None,
        fullscreen: false,
        headless: true,
        enable_devtools: false,
        log_level: shell_app::LogLevel::Info,
    };

    // When: Creating BrowserApp
    let result = BrowserApp::new(config).await;

    // Then: Should initialize browser_shell internally
    assert!(result.is_ok(), "BrowserApp should initialize successfully");
}

#[tokio::test]
async fn test_browser_app_with_fullscreen() {
    // Given: Config with fullscreen enabled
    let temp_dir = TempDir::new().unwrap();
    let config = AppConfig {
        user_data_dir: Some(temp_dir.path().to_str().unwrap().to_string()),
        initial_url: None,
        fullscreen: true,
        headless: true,
        enable_devtools: false,
        log_level: shell_app::LogLevel::Info,
    };

    // When: Creating BrowserApp
    let result = BrowserApp::new(config).await;

    // Then: Should handle fullscreen config
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_browser_app_with_initial_url() {
    // Given: Config with initial URL
    let temp_dir = TempDir::new().unwrap();
    let config = AppConfig {
        user_data_dir: Some(temp_dir.path().to_str().unwrap().to_string()),
        initial_url: Some("https://rust-lang.org".to_string()),
        fullscreen: false,
        headless: true,
        enable_devtools: false,
        log_level: shell_app::LogLevel::Info,
    };

    // When: Creating BrowserApp
    let result = BrowserApp::new(config).await;

    // Then: Should handle initial URL
    assert!(result.is_ok());
}
