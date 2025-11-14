//! Test helpers for integration and E2E tests

use browser_shell::ShellConfig;
use shared_types::WindowConfig;
use tempfile::TempDir;

/// Create a test shell configuration with temporary directory
pub fn create_test_config() -> (ShellConfig, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let config = ShellConfig {
        window_config: WindowConfig::default(),
        enable_devtools: true,
        enable_extensions: false,
        user_data_dir: temp_dir.path().to_str().unwrap().to_string(),
    };

    (config, temp_dir)
}

/// Create a default window configuration for testing
pub fn default_window_config() -> WindowConfig {
    WindowConfig::default()
}
